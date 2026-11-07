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
    void timerCallback() override { repaint(); }
    RadarVuProcessor& proc;
    juce::ComponentBoundsConstrainer constrainer;
};
