// Offscreen UI audit for RadarVU: feeds the processor a stepped stereo programme, renders the editor
// to PNG in both display modes, and runs an offscreen visual sweep that proves every parameter moves
// the GUI. Each render is preceded by a reset + full re-feed so the radar history and sweep are
// identical except for the one parameter under test. Mirrors ArcVu's qa/UiAudit.cpp. No window opens.
#include <JuceHeader.h>
#include "PluginProcessor.h"
#include <cmath>
#include <iostream>
#include <memory>

static int countChanged(const juce::Image& a, const juce::Image& b) {
    const int w = juce::jmin(a.getWidth(), b.getWidth());
    const int h = juce::jmin(a.getHeight(), b.getHeight());
    int n = 0;
    for (int y = 0; y < h; ++y) {
        for (int x = 0; x < w; ++x) {
            auto ca = a.getPixelAt(x, y);
            auto cb = b.getPixelAt(x, y);
            if (std::abs(ca.getRed() - cb.getRed()) > 24 || std::abs(ca.getGreen() - cb.getGreen()) > 24 ||
                std::abs(ca.getBlue() - cb.getBlue()) > 24)
                ++n;
        }
    }
    return n;
}

// State round trip without a host: every parameter set to a non-default value, saved with
// getStateInformation, restored into a fresh processor with setStateInformation, then compared.
static bool stateRoundTrip() {
    RadarVuProcessor a;
    const float values[6] = { 0.13f, 0.5f, 0.87f, 0.24f, 0.61f, 0.98f };
    int i = 0;
    for (auto* p : a.getParameters()) if (i < 6) p->setValueNotifyingHost(values[i++]);
    juce::MemoryBlock blob;
    a.getStateInformation(blob);
    RadarVuProcessor b;
    b.setStateInformation(blob.getData(), (int) blob.getSize());
    bool ok = true;
    for (int k = 0; k < 6; ++k) {
        auto* pa = dynamic_cast<juce::AudioParameterChoice*>(a.getParameters()[k]);
        auto* pb = dynamic_cast<juce::AudioParameterChoice*>(b.getParameters()[k]);
        const bool same = pa && pb && pa->getIndex() == pb->getIndex();
        std::cout << "  " << (same ? "PASS" : "FAIL") << "  state " << pa->getName(40) << ": saved index "
                  << pa->getIndex() << ", restored " << pb->getIndex() << "\n";
        ok = ok && same;
    }
    // A project saved in the earlier <P id index> layout must still load.
    juce::XmlElement old("RadarVU");
    const char* ids[6] = { "meter", "reference", "target", "ceiling", "window", "mode" };
    const int idx[6] = { 1, 0, 2, 1, 3, 1 };
    for (int k = 0; k < 6; ++k) { auto* e = old.createNewChildElement("P"); e->setAttribute("id", ids[k]); e->setAttribute("index", idx[k]); }
    juce::MemoryBlock oldBlob;
    juce::AudioProcessor::copyXmlToBinary(old, oldBlob);
    RadarVuProcessor c;
    c.setStateInformation(oldBlob.getData(), (int) oldBlob.getSize());
    for (int k = 0; k < 6; ++k) {
        auto* pc = dynamic_cast<juce::AudioParameterChoice*>(c.getParameters()[k]);
        const bool same = pc && pc->getIndex() == idx[k];
        std::cout << "  " << (same ? "PASS" : "FAIL") << "  old layout " << ids[k] << ": restored " << (pc ? pc->getIndex() : -1) << ", want " << idx[k] << "\n";
        ok = ok && same;
    }
    return ok;
}

int main(int argc, char** argv) {
    std::cout << "state round trip (no host):\n";
    const bool stateOk = stateRoundTrip();
    std::cout << (stateOk ? "state round trip: PASS\n" : "state round trip: FAIL\n");

    juce::File outDir = juce::File::getCurrentWorkingDirectory();
    if (argc > 1) outDir = juce::File(argv[1]);
    outDir.createDirectory();

    RadarVuProcessor proc;
    proc.prepareToPlay(48000.0, 512);

    const int sr = 48000;
    const int totalFrames = sr * 70;
    juce::AudioBuffer<float> buf(2, 512);
    juce::MidiBuffer midi;
    const double step = juce::MathConstants<double>::twoPi * 1000.0 / sr;

    auto levelAt = [](double t) -> double {
        if (t < 20) return -23.0;
        if (t < 30) return -26.0;
        if (t < 40) return -18.0;
        if (t < 50) return -30.0;
        if (t < 60) return -14.0;
        // Final window: a cosine sweep -36 .. -6 .. -36 dBFS so the radar fill spans the full heat ramp.
        const double tt = (t - 60.0) / 10.0;
        return -36.0 + 30.0 * (0.5 - 0.5 * std::cos(tt * 2.0 * juce::MathConstants<double>::pi));
    };

    double phase = 0.0;
    auto feedAll = [&]() {
        phase = 0.0;
        int frame = 0;
        while (frame < totalFrames) {
            const int n = juce::jmin(512, totalFrames - frame);
            buf.setSize(2, n, false, false, true);
            const double t0 = (double) frame / sr;
            for (int i = 0; i < n; ++i) {
                const double t = t0 + (double) i / sr;
                const float amp = (float) std::pow(10.0, levelAt(t) / 20.0);
                const float v = amp * (float) std::sin(phase);
                phase += step;
                buf.setSample(0, i, v);
                buf.setSample(1, i, v);
            }
            proc.processBlock(buf, midi);
            frame += n;
        }
    };

    std::unique_ptr<juce::AudioProcessorEditor> editor(proc.createEditor());
    editor->setSize(1100, 560);

    auto render = [&]() {
        juce::Image img(juce::Image::ARGB, 1100, 560, true);
        juce::Graphics g(img);
        editor->paintEntireComponent(g, true);
        return img;
    };

    auto save = [&](const juce::String& name, const juce::Image& img) {
        auto file = outDir.getChildFile(name);
        if (file.existsAsFile()) file.deleteFile();
        juce::FileOutputStream stream(file);
        juce::PNGImageFormat png;
        const bool ok = stream.openedOk() && png.writeImageToStream(img, stream);
        std::cout << (ok ? "wrote " : "FAILED ") << file.getFullPathName() << std::endl;
        return ok;
    };

    auto hostParams = proc.getParameters();
    auto choice = [&](int i) { return dynamic_cast<juce::AudioParameterChoice*>(hostParams[i]); };
    auto setChoice = [&](int i, int index) {
        auto* p = choice(i);
        const int n = p->choices.size();
        p->setValueNotifyingHost(n > 1 ? (float) index / (float) (n - 1) : 0.0f);
    };
    // Order: meter(0), reference(1), target(2), ceiling(3), window(4), mode(5).
    const int defaults[6] = { 0, 1, 3, 0, 1, 0 };
    const int extremes[6] = { 9, 0, 0, 2, 4, 1 };
    const char* names[6] = { "meter", "reference", "target", "ceiling", "window", "mode" };

    // Reset the meter, apply the given parameter state, and re-feed the whole programme so every
    // render is deterministic apart from the one parameter under test.
    auto renderState = [&](const int* vals) {
        proc.resetLoudness();
        for (int j = 0; j < 6; ++j) setChoice(j, vals[j]);
        feedAll();
        return render();
    };

    bool ok = save("radarvu-radar.png", renderState(defaults));
    int spectrumVals[6];
    for (int j = 0; j < 6; ++j) spectrumVals[j] = defaults[j];
    spectrumVals[5] = 1; // spectrum mode
    ok = save("radarvu-spectrum.png", renderState(spectrumVals)) && ok;

    juce::Image baseline = renderState(defaults);
    int failures = 0;
    std::cout << "offscreen visual sweep (baseline = defaults):" << std::endl;
    for (int i = 0; i < 6; ++i) {
        int vals[6];
        for (int j = 0; j < 6; ++j) vals[j] = defaults[j];
        vals[i] = extremes[i];
        juce::Image changed = renderState(vals);
        const int diff = countChanged(baseline, changed);
        save(juce::String("sweep-") + names[i] + ".png", changed);
        const bool shown = diff > 0;
        if (!shown) ++failures;
        std::cout << (shown ? "  PASS  " : "  FAIL  ") << names[i] << " changes " << diff
                  << " pixels (" << (100.0 * diff / (1100.0 * 560.0)) << "%)" << std::endl;
    }

    std::cout << (failures == 0 ? "visual sweep: all parameters shown in the GUI" : "visual sweep: FAILURES")
              << std::endl;

    // Fill and FFT toggles (UI-only, not host parameters).
    int toggleFailures = 0;
    auto checkToggle = [&](const char* name, const juce::Image& on, const juce::Image& off) {
        const int diff = countChanged(on, off);
        const bool shown = diff > 0;
        if (!shown) ++toggleFailures;
        std::cout << (shown ? "  PASS  " : "  FAIL  ") << name << " toggle changes " << diff << " pixels" << std::endl;
    };

    proc.setFill(true);
    juce::Image fillOn = renderState(defaults);
    proc.setFill(false);
    juce::Image fillOff = render();
    proc.setFill(true);
    save("sweep-fill-off.png", fillOff);
    checkToggle("fill", fillOn, fillOff);

    int specVals[6];
    for (int j = 0; j < 6; ++j) specVals[j] = defaults[j];
    specVals[5] = 1; // spectrum mode
    proc.setFft(true);
    juce::Image fftOn = renderState(specVals);
    proc.setFft(false);
    juce::Image fftOff = render();
    proc.setFft(true);
    save("sweep-fft-off.png", fftOff);
    checkToggle("fft", fftOn, fftOff);

    std::cout << (toggleFailures == 0 ? "toggles: fill and fft switch the GUI" : "toggles: FAILURES") << std::endl;
    return (ok && failures == 0 && toggleFailures == 0) ? 0 : 1;
}
