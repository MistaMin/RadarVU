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
    void showChoiceMenu(int parameterIndex, juce::Rectangle<int> anchor);
    // Presets plus "Custom value..." for the target (parameter 2) and the true-peak ceiling (3).
    void showValueMenu(int parameterIndex, juce::Point<int> screenPosition);
    void askForValue(int parameterIndex);
    std::shared_ptr<juce::LookAndFeel_V4> makeMenuLook() const;
    void timerCallback() override { repaint(); }
    RadarVuProcessor& proc;
    juce::ComponentBoundsConstrainer constrainer;
};
