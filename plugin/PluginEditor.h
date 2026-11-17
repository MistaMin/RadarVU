#pragma once
#include <JuceHeader.h>
#include "PluginProcessor.h"

class RadarVuEditor : public juce::AudioProcessorEditor, private juce::Timer {
public:
    explicit RadarVuEditor(RadarVuProcessor&);
    void paint(juce::Graphics&) override;
    void resized() override {}
    void mouseDown(const juce::MouseEvent&) override;
    void mouseUp(const juce::MouseEvent&) override;

private:
    void showChoiceMenu(const juce::String& parameterId, juce::Rectangle<int> anchor);
    // Presets plus "Custom value..." for the target and the true-peak ceiling (parameter IDs "target", "ceiling").
    void showValueMenu(const juce::String& parameterId, juce::Point<int> screenPosition);
    void askForValue(const juce::String& parameterId);
    std::shared_ptr<juce::LookAndFeel_V4> makeMenuLook() const;
    void timerCallback() override { repaint(); }
    RadarVuProcessor& proc;
    std::vector<RadarVuPaint> cmds = std::vector<RadarVuPaint>(8192); // reused by every paint
    juce::ComponentBoundsConstrainer constrainer;
};
