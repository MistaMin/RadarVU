// Real-music rendering and paint timing. Media and derived frames stay in the supplied scratch dir.
// Usage: RadarVU_QA OUTPUT --music FILE [window index 0..4] [block size] [width]
//                    [frames|no-frames] [VST3 path or -] [resolution index 0..3] [skin index 0..5]
// A native VST3 view cannot be rendered offscreen here: hosted runs verify loading, audio, state
// and editor creation only. Use direct runs for screenshots, paint timing and command coverage.
#pragma once
#include <algorithm>
#include <chrono>
#include <cstdlib>

static int auditMusic(int argc, char** argv) {
    juce::ScopedJuceInitialiser_GUI gui;
    const juce::File outDir(argv[1]);
    outDir.createDirectory();
    const juce::File source(argv[3]);
    const int windowIndex = argc > 4 ? std::atoi(argv[4]) : 1;
    const int block = argc > 5 ? std::atoi(argv[5]) : 512;
    const int width = argc > 6 ? std::atoi(argv[6]) : 1100;
    const bool frames = argc > 7 && juce::String(argv[7]) == "frames";
    const int resolution = argc > 9 ? std::atoi(argv[9]) : 2;
    const int skin = argc > 10 ? std::atoi(argv[10]) : 0;
    if (windowIndex < 0 || windowIndex > 4 || block < 1 || width < 440 || width > 4000
        || resolution < 0 || resolution > 3 || skin < 0 || skin > 5) return 2;
    juce::AudioFormatManager formats;
    formats.registerBasicFormats();
    std::unique_ptr<juce::AudioFormatReader> reader(formats.createReaderFor(source));
    if (!reader || reader->numChannels != 2) {
        std::cerr << "Expected readable stereo music: " << source.getFullPathName() << std::endl;
        return 2;
    }
    const int sr = (int) reader->sampleRate;
    const auto start = juce::jmin<juce::int64>(sr * 20, reader->lengthInSamples / 4);
    const int count = (int) juce::jmin<juce::int64>(sr * 45, reader->lengthInSamples - start);
    if (sr <= 0 || count <= 0) return 2;
    juce::AudioBuffer<float> music(2, count);
    if (!reader->read(&music, 0, count, start, true, true)) return 2;

    RadarVuProcessor direct;
    std::unique_ptr<juce::AudioPluginInstance> hosted;
    if (argc > 8 && juce::String(argv[8]) != "-") {
        juce::VST3PluginFormat format;
        juce::OwnedArray<juce::PluginDescription> descriptions;
        format.findAllTypesForFile(descriptions, argv[8]);
        juce::String error;
        if (!descriptions.isEmpty())
            hosted = format.createInstanceFromDescription(*descriptions[0], sr, block, error);
        if (!hosted) {
            std::cerr << "VST3 load failed: " << error << std::endl;
            return 2;
        }
    }
    juce::AudioProcessor& proc = hosted ? static_cast<juce::AudioProcessor&>(*hosted) : direct;
    proc.setRateAndBufferSizeDetails(sr, block);
    proc.prepareToPlay(sr, block);
    auto choices = proc.getParameters();
    auto setParameter = [&](const juce::String& name, float value) {
        for (auto* p : choices) if (p->getName(60) == name) {
            p->setValueNotifyingHost(value);
            return true;
        }
        std::cerr << "Missing host parameter: " << name << std::endl;
        return false;
    };
    if (!setParameter("Meter type", 1.0f) || !setParameter("Sweep window", windowIndex / 4.0f)) return 2;
    if (!setParameter("Radar resolution", resolution / 3.0f) || !setParameter("Colour skin", skin / 5.0f)) return 2;
    std::unique_ptr<juce::AudioProcessorEditor> editor(proc.createEditorAndMakeActive());
    if (!editor) return 2;
    const int height = juce::roundToInt(width * 560.0 / 1100.0);
    editor->setSize(width, height);
    juce::AudioBuffer<float> buf(2, block);
    juce::MidiBuffer midi;
    juce::Image image(juce::Image::ARGB, width, height, true);
    std::vector<RadarVuPaint> commands(8192);
    std::vector<double> paintMs;
    double processMs = 0.0;
    float maxAudioDifference = 0.0f;
    bool finiteAudio = true;
    int maxCommands = 0, maxPoints = 0, frame = 0, renderIndex = 0;
    const int renderStart = juce::jmax(0, count - sr * 10);
    int nextRender = renderStart;
    juce::String readouts;
    using Timer = std::chrono::steady_clock;
    auto ms = [](auto a, auto b) { return std::chrono::duration<double, std::milli>(b - a).count(); };
    auto save = [&](const juce::File& file) {
        file.deleteFile();
        juce::FileOutputStream stream(file);
        return stream.openedOk() && juce::PNGImageFormat().writeImageToStream(image, stream);
    };
    bool saved = true;
    while (frame < count) {
        const int end = juce::jmin(count, nextRender > frame ? nextRender : count);
        const int n = juce::jmin(block, end - frame);
        buf.setSize(2, n, false, false, true);
        for (int c = 0; c < 2; ++c) buf.copyFrom(c, 0, music, c, frame, n);
        auto t0 = Timer::now();
        proc.processBlock(buf, midi);
        processMs += ms(t0, Timer::now());
        for (int c = 0; c < 2; ++c)
            for (int i = 0; i < n; ++i) {
                finiteAudio = finiteAudio && std::isfinite(buf.getSample(c, i));
                maxAudioDifference = juce::jmax(maxAudioDifference,
                    std::abs(buf.getSample(c, i) - music.getSample(c, frame + i)));
            }
        frame += n;
        if (frame >= nextRender && frame >= renderStart) {
            auto t1 = Timer::now();
            { juce::Graphics g(image); editor->paintEntireComponent(g, true); }
            paintMs.push_back(ms(t1, Timer::now()));
            const int nc = hosted ? 0 : direct.paint(commands.data(), (int) commands.size());
            maxCommands = juce::jmax(maxCommands, nc);
            readouts.clear();
            for (int i = 0; i < nc; ++i) {
                maxPoints = juce::jmax(maxPoints, commands[i].npts);
                if (commands[i].kind == 10)
                    readouts += juce::String::fromUTF8((const char*) commands[i].text) + " | ";
            }
            if (frames && !hosted) saved = save(outDir.getChildFile("frame-" + juce::String(renderIndex).paddedLeft('0', 4) + ".png")) && saved;
            ++renderIndex;
            nextRender = renderStart + juce::roundToInt(renderIndex * (double) sr / 30.0);
        }
    }
    if (!hosted) saved = save(outDir.getChildFile("music-radar.png")) && saved;
    if (!hosted) {
        direct.setFill(false);
        { juce::Graphics g(image); editor->paintEntireComponent(g, true); }
        saved = save(outDir.getChildFile("music-outline.png")) && saved;
        direct.setFill(true);
    }
    if (!setParameter("Display mode", 1.0f)) return 2;
    { juce::Graphics g(image); editor->paintEntireComponent(g, true); }
    if (!hosted) saved = save(outDir.getChildFile("music-spectrum.png")) && saved;

    juce::MemoryBlock state;
    proc.getStateInformation(state);
    std::vector<float> values;
    for (auto* p : choices) values.push_back(p->getValue());
    setParameter("Meter type", 0.0f);
    setParameter("Sweep window", 0.0f);
    setParameter("Radar resolution", resolution == 0 ? 1.0f : 0.0f);
    setParameter("Colour skin", skin == 0 ? 1.0f : 0.0f);
    proc.setStateInformation(state.getData(), (int) state.getSize());
    bool stateOk = true;
    for (int i = 0; i < choices.size(); ++i)
        stateOk = stateOk && std::abs(choices[i]->getValue() - values[(size_t) i]) < 1.0e-6f;
    // The host must notify JUCE before destroying an active editor (the format wrappers do this).
    proc.editorBeingDeleted(editor.get());
    editor.reset();
    editor.reset(proc.createEditorAndMakeActive());
    const bool lifecycleOk = editor != nullptr;
    if (editor) proc.editorBeingDeleted(editor.get());

    std::sort(paintMs.begin(), paintMs.end());
    double sum = 0.0;
    for (auto t : paintMs) sum += t;
    const bool ok = saved && stateOk && lifecycleOk && finiteAudio && maxAudioDifference == 0.0f
        && maxCommands < 8192 && maxPoints <= 32 && !paintMs.empty();
    juce::DynamicObject::Ptr report = new juce::DynamicObject();
    report->setProperty("source", source.getFullPathName());
    report->setProperty("plugin_binary", hosted ? juce::String(argv[8]) : "direct processor/editor");
    juce::Array<juce::var> parameterNames;
    for (auto* p : choices) parameterNames.add(p->getName(60));
    report->setProperty("parameter_names", juce::var(parameterNames));
    report->setProperty("start_seconds", start / (double) sr);
    report->setProperty("duration_seconds", count / (double) sr);
    report->setProperty("sample_rate", sr);
    report->setProperty("block_size", block);
    report->setProperty("window_index", windowIndex);
    report->setProperty("resolution_index", resolution);
    report->setProperty("skin_index", skin);
    report->setProperty("width", width);
    report->setProperty("frames_painted", hosted ? 0 : (int) paintMs.size());
    report->setProperty("gui_check", hosted ? "SKIPPED: native VST3 view cannot be captured by offscreen component rendering" : "PASS: actual editor rendered offscreen");
    report->setProperty("paint_mean_ms", hosted ? juce::var() : juce::var(sum / juce::jmax(1, (int) paintMs.size())));
    report->setProperty("paint_p95_ms", hosted ? juce::var() : juce::var(paintMs[(size_t) (0.95 * (paintMs.size() - 1))]));
    report->setProperty("state_round_trip", stateOk);
    report->setProperty("editor_recreated", lifecycleOk);
    report->setProperty("process_total_ms", processMs);
    report->setProperty("max_audio_difference", maxAudioDifference);
    report->setProperty("finite_audio", finiteAudio);
    report->setProperty("max_commands", maxCommands);
    report->setProperty("max_points_per_command", maxPoints);
    report->setProperty("readouts", readouts);
    report->setProperty("pass", ok);
    const auto json = juce::JSON::toString(juce::var(report.get()));
    outDir.getChildFile("music-report.json").replaceWithText(json);
    std::cout << json << std::endl;
    return ok ? 0 : 1;
}
