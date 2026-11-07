#include "PluginProcessor.h"
#include "PluginEditor.h"

namespace {
// Earlier versions offered only these values as choice indices; kept to migrate old projects.
constexpr float LEGACY_TARGETS[4] = { -23.0f, -18.0f, -16.0f, -14.0f };
constexpr float LEGACY_CEILINGS[3] = { -1.0f, -2.0f, -3.0f };

int nearestLegacyIndex(const float* table, int n, float value) {
    int best = 0;
    for (int i = 1; i < n; ++i)
        if (std::abs(table[i] - value) < std::abs(table[best] - value)) best = i;
    return best;
}

std::unique_ptr<juce::AudioParameterFloat> makeLevelParameter(const char* id, const char* name, float lo, float hi,
                                                              float fallback, const juce::String& unit) {
    juce::NormalisableRange<float> range(lo, hi, 0.1f);
    auto attributes = juce::AudioParameterFloatAttributes()
        .withLabel(unit)
        .withStringFromValueFunction([unit](float v, int) { return juce::String(v, 1) + " " + unit; })
        .withValueFromStringFunction([lo, hi, fallback](const juce::String& text) {
            const auto t = text.retainCharacters("0123456789.-+\u2212").replace(juce::String::charToString(0x2212), "-");
            return t.containsAnyOf("0123456789") ? juce::jlimit(lo, hi, t.getFloatValue()) : fallback;
        });
    return std::make_unique<juce::AudioParameterFloat>(id, name, range, fallback, attributes);
}
constexpr float WINDOWS[5] = { 5.0f, 10.0f, 15.0f, 20.0f, 30.0f };
constexpr float REFERENCES[3] = { -20.0f, -18.0f, -14.0f };
constexpr int HISTORY_POINTS[4] = { 180, 360, 720, 1440 };
}

RadarVuProcessor::RadarVuProcessor()
    : juce::AudioProcessor(BusesProperties()
          .withInput("Input", juce::AudioChannelSet::stereo(), true)
          .withOutput("Output", juce::AudioChannelSet::stereo(), true)) {
    meter = radarvu_create(48000.0);

    meterTypeP = new juce::AudioParameterChoice("meter", "Meter type",
        { "VU", "RMS", "K-12", "K-14", "K-20", "DIN", "Nordic", "BBC", "EBU", "Loudness" }, 0);
    addParameter(meterTypeP);
    referenceP = new juce::AudioParameterChoice("reference", "Reference level",
        { "-20 dBFS", "-18 dBFS", "-14 dBFS" }, 1);
    addParameter(referenceP);
    targetP = makeLevelParameter("target", "Target level", targetMin, targetMax, -14.0f, "LUFS").release();
    addParameter(targetP);
    ceilingP = makeLevelParameter("ceiling", "True-peak ceiling", ceilingMin, ceilingMax, -1.0f, "dBTP").release();
    addParameter(ceilingP);
    windowP = new juce::AudioParameterChoice("window", "Sweep window",
        { "5 s", "10 s", "15 s", "20 s", "30 s" }, 1);
    addParameter(windowP);
    modeP = new juce::AudioParameterChoice("mode", "Display mode",
        { "Radar", "Spectrum" }, 0);
    addParameter(modeP);
    resolutionP = new juce::AudioParameterChoice("resolution", "Radar resolution",
        { "180 points", "360 points", "720 points", "1440 points" }, 2);
    addParameter(resolutionP);
    skinP = new juce::AudioParameterChoice("skin", "Colour skin",
        { "Stock Dark", "Retro Console", "Ivory Terminal", "Matrix", "Modern Console", "Studio Amber" }, 0);
    addParameter(skinP);

    for (auto* p : params()) p->addListener(this);
    interleaved.reserve(4096 * 2);
    pullSetup();
}

RadarVuProcessor::~RadarVuProcessor() {
    for (auto* p : params()) p->removeListener(this);
    if (meter) { radarvu_destroy(meter); meter = nullptr; }
}

void RadarVuProcessor::readParams(RadarVuSetup& s) const {
    s.mode = modeP->getIndex();
    s.target = targetP->get();
    s.ceiling = ceilingP->get();
    s.window = WINDOWS[windowP->getIndex()];
    s.kind = meterTypeP->getIndex();
    s.reference_db = REFERENCES[referenceP->getIndex()];
    s.fill_on = fillOn ? 1 : 0;
    s.fft_on = fftOn ? 1 : 0;
    s.history_points = HISTORY_POINTS[resolutionP->getIndex()];
    s.skin = skinP->getIndex();
}

void RadarVuProcessor::pullSetup() {
    if (!meter) return;
    RadarVuSetup s {};
    readParams(s);
    radarvu_setup(meter, &s);
}

void RadarVuProcessor::setTarget(float lufs) {
    targetP->beginChangeGesture();
    targetP->setValueNotifyingHost(targetP->convertTo0to1(juce::jlimit(targetMin, targetMax, lufs)));
    targetP->endChangeGesture();
    pullSetup();
}

void RadarVuProcessor::setCeiling(float dbtp) {
    ceilingP->beginChangeGesture();
    ceilingP->setValueNotifyingHost(ceilingP->convertTo0to1(juce::jlimit(ceilingMin, ceilingMax, dbtp)));
    ceilingP->endChangeGesture();
    pullSetup();
}

void RadarVuProcessor::cycle(int which) {
    if (which != 0 && which != 1 && which != 4) return; // target and ceiling are set by value, not stepped
    juce::AudioParameterChoice* p = which == 0 ? meterTypeP : which == 1 ? referenceP : windowP;
    const int n = p->choices.size();
    if (n < 2) return;
    const int next = (p->getIndex() + 1) % n;
    p->setValueNotifyingHost((float) next / (float) (n - 1));
    pullSetup();
}

void RadarVuProcessor::setMode(int index) {
    const int n = modeP->choices.size();
    modeP->setValueNotifyingHost(n > 1 ? (float) juce::jlimit(0, n - 1, index) / (float) (n - 1) : 0.0f);
    pullSetup();
}

void RadarVuProcessor::prepareToPlay(double sampleRate, int) {
    radarvu_set_sample_rate(meter, sampleRate);
    pullSetup();
}

bool RadarVuProcessor::isBusesLayoutSupported(const BusesLayout& layouts) const {
    if (layouts.getMainOutputChannelSet() != layouts.getMainInputChannelSet()) return false;
    const int ch = layouts.getMainInputChannels();
    return ch >= 1 && ch <= 2;
}

void RadarVuProcessor::processBlock(juce::AudioBuffer<float>& buffer, juce::MidiBuffer&) {
    juce::ScopedNoDenormals noDenormals;

    const int totalIn = getTotalNumInputChannels();
    const int totalOut = getTotalNumOutputChannels();
    for (int i = totalIn; i < totalOut; ++i) buffer.clear(i, 0, buffer.getNumSamples());

    const int frames = buffer.getNumSamples();
    const int channels = juce::jlimit(1, 2, totalIn);
    interleaved.resize((size_t) frames * 2);

    auto* inL = buffer.getReadPointer(0);
    auto* inR = channels > 1 ? buffer.getReadPointer(1) : inL;
    for (int i = 0; i < frames; ++i) {
        interleaved[(size_t) i * 2] = inL[i];
        interleaved[(size_t) i * 2 + 1] = inR[i];
    }
    radarvu_process(meter, interleaved.data(), frames, channels);

    // RadarVU is a meter: the audio passes through unchanged.
}

void RadarVuProcessor::getStateInformation(juce::MemoryBlock& destData) {
    auto root = std::make_unique<juce::XmlElement>("RadarVU");
    for (auto* p : params()) {
        // Standard layout (<PARAM id value>) so hosts and test tools can read the real state;
        // "index" is kept for readers of the earlier <P> layout.
        auto* e = root->createNewChildElement("PARAM");
        e->setAttribute("id", p->getParameterID());
        if (auto* c = dynamic_cast<juce::AudioParameterChoice*>(p)) {
            e->setAttribute("value", c->getIndex());
        } else if (auto* f = dynamic_cast<juce::AudioParameterFloat*>(p)) {
            // "real" is the exact value; "value" is the nearest earlier choice index for old readers.
            const bool isTarget = f == targetP;
            e->setAttribute("real", (double) f->get());
            e->setAttribute("value", isTarget ? nearestLegacyIndex(LEGACY_TARGETS, 4, f->get())
                                              : nearestLegacyIndex(LEGACY_CEILINGS, 3, f->get()));
        }
    }
    copyXmlToBinary(*root, destData);
}

void RadarVuProcessor::setStateInformation(const void* data, int sizeInBytes) {
    if (auto root = getXmlFromBinary(data, sizeInBytes)) {
        if (!root->hasTagName("RadarVU")) return;
        // Projects made before these settings existed always reopen with their original defaults.
        resolutionP->setValueNotifyingHost(2.0f / 3.0f);
        skinP->setValueNotifyingHost(0.0f);
        for (auto* e : root->getChildIterator()) {
            const bool old = e->hasTagName("P");   // earlier layout: <P id index>
            if (!old && !e->hasTagName("PARAM")) continue;
            const auto id = e->getStringAttribute("id");
            for (auto* p : params()) {
                if (p->getParameterID() != id) continue;
                const int storedIndex = old ? e->getIntAttribute("index") : e->getIntAttribute("value");
                if (auto* c = dynamic_cast<juce::AudioParameterChoice*>(p)) {
                    const int n = c->choices.size();
                    if (n > 1) c->setValueNotifyingHost((float) juce::jlimit(0, n - 1, storedIndex) / (float) (n - 1));
                } else if (auto* f = dynamic_cast<juce::AudioParameterFloat*>(p)) {
                    // Projects saved before these were free values only hold a choice index.
                    float value;
                    if (e->hasAttribute("real")) value = (float) e->getDoubleAttribute("real");
                    else if (f == targetP) value = LEGACY_TARGETS[juce::jlimit(0, 3, storedIndex)];
                    else value = LEGACY_CEILINGS[juce::jlimit(0, 2, storedIndex)];
                    f->setValueNotifyingHost(f->convertTo0to1(juce::jlimit(f->range.start, f->range.end, value)));
                }
            }
        }
        pullSetup();
    }
}

juce::AudioProcessorEditor* RadarVuProcessor::createEditor() {
    return new RadarVuEditor(*this);
}

juce::AudioProcessor* JUCE_CALLTYPE createPluginFilter() {
    return new RadarVuProcessor();
}
