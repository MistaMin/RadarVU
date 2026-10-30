#include "PluginEditor.h"
#include "radarvu.h"
#include <cmath>
#include <vector>

namespace {
juce::Colour ink(unsigned int argb) { return juce::Colour(argb); }

void fillGradient(juce::Graphics& g, juce::Rectangle<float> r, unsigned int c0, unsigned int c1, bool diagonal, bool vertical) {
    if (c0 == c1) { g.setColour(ink(c0)); return; }
    auto a = ink(c0);
    auto b = ink(c1);
    if (diagonal)
        g.setGradientFill(juce::ColourGradient(a, r.getX(), r.getY(), b, r.getRight(), r.getBottom(), false));
    else if (vertical)
        g.setGradientFill(juce::ColourGradient(a, r.getX(), r.getY(), b, r.getX(), r.getBottom(), false));
    else
        g.setGradientFill(juce::ColourGradient(a, r.getX(), r.getY(), b, r.getRight(), r.getY(), false));
}

// Replays the drawing-command list from the Rust detector (kinds mirror GoodLookinDri's Scene).
void replay(juce::Graphics& g, const RadarVuPaint& cmd) {
    auto rect = juce::Rectangle<float>(cmd.x, cmd.y, cmd.w, cmd.h);
    switch (cmd.kind) {
        case 1:
            fillGradient(g, rect, cmd.c0, cmd.c1, cmd.flags != 0, false);
            g.fillEllipse(rect);
            break;
        case 2:
            g.setColour(ink(cmd.c0));
            g.drawEllipse(rect, cmd.width);
            break;
        case 3:
            fillGradient(g, rect, cmd.c0, cmd.c1, false, cmd.flags != 0);
            g.fillRoundedRectangle(rect, cmd.radius);
            break;
        case 4:
            g.setColour(ink(cmd.c0));
            g.drawRoundedRectangle(rect, cmd.radius, cmd.width);
            break;
        case 5:
            fillGradient(g, rect, cmd.c0, cmd.c1, false, cmd.flags != 0);
            g.fillRect(rect);
            break;
        case 6:
            g.setColour(ink(cmd.c0));
            g.drawLine(cmd.x, cmd.y, cmd.x2, cmd.y2, cmd.width);
            break;
        case 7:
        case 8: {
            juce::Path path;
            if (cmd.npts > 0) path.startNewSubPath(cmd.pts[0], cmd.pts[1]);
            for (int i = 1; i < cmd.npts; ++i) path.lineTo(cmd.pts[i * 2], cmd.pts[i * 2 + 1]);
            if (cmd.kind == 7 || cmd.flags) path.closeSubPath();
            if (cmd.kind == 7) {
                auto bounds = path.getBounds();
                fillGradient(g, bounds, cmd.c0, cmd.c1, cmd.flags != 0, false);
                g.fillPath(path);
            } else {
                g.setColour(ink(cmd.c0));
                g.strokePath(path, juce::PathStrokeType(cmd.width));
            }
            break;
        }
        case 9: {
            juce::Path path;
            const int steps = 24;
            for (int i = 0; i <= steps; ++i) {
                const float t = (float) i / (float) steps;
                const float a = cmd.x2 + (cmd.y2 - cmd.x2) * t;
                const float px = cmd.x + std::cos(a) * cmd.radius;
                const float py = cmd.y + std::sin(a) * cmd.radius;
                if (i == 0) path.startNewSubPath(px, py);
                else path.lineTo(px, py);
            }
            g.setColour(ink(cmd.c0));
            g.strokePath(path, juce::PathStrokeType(cmd.width));
            break;
        }
        case 10: {
            g.setColour(ink(cmd.c0));
            const char* face = cmd.w >= 15.0f ? "Avenir Next" : "Avenir Next Condensed";
            g.setFont(juce::Font(juce::FontOptions(face, cmd.w, juce::Font::plain)));
            auto text = juce::String::fromUTF8((const char*) cmd.text);
            const float box = juce::jmax(cmd.w * 5.0f, 200.0f);
            if (cmd.flags)
                g.drawText(text, juce::Rectangle<float>(cmd.x - box * 0.5f, cmd.y - cmd.w * 0.65f, box, cmd.w * 1.35f),
                           juce::Justification::centred);
            else
                g.drawText(text, juce::Rectangle<float>(cmd.x, cmd.y - cmd.w, 220.0f, cmd.w * 1.45f),
                           juce::Justification::left);
            break;
        }
        default:
            break;
    }
}
}

RadarVuEditor::RadarVuEditor(RadarVuProcessor& p) : AudioProcessorEditor(p), proc(p) {
    constrainer.setMinimumSize(440, 224);
    constrainer.setMaximumSize(4000, 4000);
    constrainer.setFixedAspectRatio(1100.0 / 560.0);
    setConstrainer(&constrainer);
    setResizable(true, true);
    setSize(1100, 560);
    startTimerHz(30);
}

void RadarVuEditor::paint(juce::Graphics& g) {
    g.fillAll(juce::Colour(0xff07090c));
    g.addTransform(juce::AffineTransform::scale(getWidth() / 1100.0f, getHeight() / 560.0f));
    std::vector<RadarVuPaint> cmds(8192);
    const int n = proc.paint(cmds.data(), (int) cmds.size());
    for (int i = 0; i < n; ++i) replay(g, cmds[(size_t) i]);
}

void RadarVuEditor::mouseDown(const juce::MouseEvent& e) {
    auto p = e.position;
    p.x *= 1100.0f / (float) getWidth();
    p.y *= 560.0f / (float) getHeight();
    switch (radarvu_click(p.x, p.y)) {
        case 1: proc.cycle(0); break; // meter type
        case 2: proc.cycle(1); break; // reference
        case 3: proc.cycle(2); break; // target
        case 4: proc.cycle(3); break; // ceiling
        case 5: proc.cycle(4); break; // window
        case 6: proc.toggleFill(); break;
        case 7: proc.toggleFft(); break;
        case 8: proc.resetLoudness(); break;
        case 9: proc.setMode(0); break; // radar
        case 10: proc.setMode(1); break; // spectrum
        default: break;
    }
}

void RadarVuEditor::mouseUp(const juce::MouseEvent&) { repaint(); }
