#pragma once
#include <JuceHeader.h>
#include "radarvu.h"
#include <array>
#include <vector>

// AAX reads its control-surface page table from Contents/Resources/RadarVU.xml.
struct RadarVuAAXExtensions : juce::AAXClientExtensions {
    juce::String getPageFileName() const override { return "RadarVU.xml"; }
};

class RadarVuProcessor : public juce::AudioProcessor,
                         private juce::AudioProcessorParameter::Listener {
public:
    RadarVuProcessor();
    ~RadarVuProcessor() override;

    void prepareToPlay(double sampleRate, int samplesPerBlock) override;
    void releaseResources() override {}
    void processBlock(juce::AudioBuffer<float>&, juce::MidiBuffer&) override;
    bool isBusesLayoutSupported(const BusesLayout& layouts) const override;

    juce::AudioProcessorEditor* createEditor() override;
    bool hasEditor() const override { return true; }
    const juce::String getName() const override { return "RadarVU"; }
    bool acceptsMidi() const override { return false; }
    bool producesMidi() const override { return false; }
    double getTailLengthSeconds() const override { return 0.0; }
    int getNumPrograms() override { return 1; }
    int getCurrentProgram() override { return 0; }
    juce::AAXClientExtensions& getAAXClientExtensions() override { return aaxExtensions; }
    void setCurrentProgram(int) override {}
    const juce::String getProgramName(int) override { return {}; }
    void changeProgramName(int, const juce::String&) override {}
    void getStateInformation(juce::MemoryBlock& destData) override;
    void setStateInformation(const void* data, int sizeInBytes) override;

    int paint(RadarVuPaint* out, int cap) const { return radarvu_paint(meter, out, cap); }
    void resetLoudness() { radarvu_reset(meter); }
    // Cycle one of the choice parameters: 0 = meter type, 1 = reference, 4 = window.
    void cycle(int which);
    // Loudness target (LUFS) and true-peak ceiling (dBTP) take any value in their range.
    static constexpr float targetMin = -40.0f, targetMax = -6.0f, ceilingMin = -6.0f, ceilingMax = 0.0f;
    void setTarget(float lufs);
    void setCeiling(float dbtp);
    void setMode(int index);
    void toggleFill() { fillOn = !fillOn; pullSetup(); }
    void toggleFft() { fftOn = !fftOn; pullSetup(); }
    bool fill() const { return fillOn; }
    bool fft() const { return fftOn; }
    void setFill(bool on) { fillOn = on; pullSetup(); }
    void setFft(bool on) { fftOn = on; pullSetup(); }
    void pullSetup();

private:
    void parameterValueChanged(int, float) override { pullSetup(); }
    void parameterGestureChanged(int, bool) override {}

    void readParams(RadarVuSetup& s) const;
    std::array<juce::RangedAudioParameter*, 8> params() const { return { meterTypeP, referenceP, targetP, ceilingP, windowP, modeP, resolutionP, skinP }; }

    RadarVuAAXExtensions aaxExtensions;
    Meter* meter = nullptr;
    juce::AudioParameterChoice* meterTypeP = nullptr;
    juce::AudioParameterChoice* referenceP = nullptr;
    juce::AudioParameterFloat* targetP = nullptr;
    juce::AudioParameterFloat* ceilingP = nullptr;
    juce::AudioParameterChoice* windowP = nullptr;
    juce::AudioParameterChoice* modeP = nullptr;
    juce::AudioParameterChoice* resolutionP = nullptr;
    juce::AudioParameterChoice* skinP = nullptr;
    std::vector<float> interleaved; // reused every block so processBlock never allocates
    bool fillOn = true;
    bool fftOn = true;
};
