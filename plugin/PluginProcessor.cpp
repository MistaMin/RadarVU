#include "PluginProcessor.h"
#include "PluginEditor.h"

namespace {
constexpr float TARGETS[4] = { -23.0f, -18.0f, -16.0f, -14.0f };
constexpr float CEILINGS[3] = { -1.0f, -2.0f, -3.0f };
constexpr float WINDOWS[5] = { 5.0f, 10.0f, 15.0f, 20.0f, 30.0f };
constexpr float REFERENCES[3] = { -20.0f, -18.0f, -14.0f };
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
    targetP = new juce::AudioParameterChoice("target", "Target level",
        { "-23 LUFS", "-18 LUFS", "-16 LUFS", "-14 LUFS" }, 3);
    addParameter(targetP);
    ceilingP = new juce::AudioParameterChoice("ceiling", "True-peak ceiling",
        { "-1 dBTP", "-2 dBTP", "-3 dBTP" }, 0);
    addParameter(ceilingP);
    windowP = new juce::AudioParameterChoice("window", "Sweep window",
        { "5 s", "10 s", "15 s", "20 s", "30 s" }, 1);
    addParameter(windowP);
    modeP = new juce::AudioParameterChoice("mode", "Display mode",
        { "Radar", "Spectrum" }, 0);
    addParameter(modeP);

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
    s.target = TARGETS[targetP->getIndex()];
    s.ceiling = CEILINGS[ceilingP->getIndex()];
    s.window = WINDOWS[windowP->getIndex()];
    s.kind = meterTypeP->getIndex();
    s.reference_db = REFERENCES[referenceP->getIndex()];
    s.fill_on = fillOn ? 1 : 0;
    s.fft_on = fftOn ? 1 : 0;
}

void RadarVuProcessor::pullSetup() {
    if (!meter) return;
    RadarVuSetup s {};
    readParams(s);
    radarvu_setup(meter, &s);
}

void RadarVuProcessor::cycle(int which) {
    juce::AudioParameterChoice* p = which == 0 ? meterTypeP : which == 1 ? referenceP
                                   : which == 2 ? targetP : which == 3 ? ceilingP : windowP;
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
        auto* e = root->createNewChildElement("P");
        e->setAttribute("id", p->getParameterID());
        e->setAttribute("index", p->getIndex());
    }
    copyXmlToBinary(*root, destData);
}

void RadarVuProcessor::setStateInformation(const void* data, int sizeInBytes) {
    if (auto root = getXmlFromBinary(data, sizeInBytes)) {
        for (auto* e : root->getChildIterator()) {
            if (!e->hasTagName("P")) continue;
            const auto id = e->getStringAttribute("id");
            for (auto* p : params()) {
                if (p->getParameterID() != id) continue;
                const int n = p->choices.size();
                const int index = juce::jlimit(0, n - 1, e->getIntAttribute("index"));
                if (n > 1) p->setValueNotifyingHost((float) index / (float) (n - 1));
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
