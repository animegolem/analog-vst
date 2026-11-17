# Seven Band Graphic — experimental

An independent component-level model of the Music Thing Modular Graphic EQ, designed by Tom Whitwell / Music Thing Modular. Trademarks belong to their owners; this project is not endorsed by the maker. This is not a verified hardware replica: the fader resistance and taper are not specified in either the schematic or the Rev 1 BOM and are **assumed** to be 10 kΩ linear. This materially affects frequency and boost/cut range.

## Sources and licensing

- [Published schematic](https://musicthing.co.uk/collateral/graphic_schematic.pdf), compared visually to the Rev 1 BOM.
- [Graphic repository and Rev 1 BOM](https://github.com/TomWhitwell/Graphic), commit `81ee3c67aaca1dc377d09ab53c3737f50a519d74`. The repository contains no LICENSE file and its README does not grant a license. The [module page](https://www.musicthing.co.uk/Graphic-EQ/) does not state a module-specific license. No Creative Commons grant for this particular module has been verified. Only independently transcribed circuit facts are used; neither the drawing nor BOM is redistributed.
- The plugin scaffold is MIT; VST3 binary distribution uses GPLv3 bindings (see GUIDE.md). This is not legal advice.

## Circuit assumptions

Seven bands, input and output op-amps and seven filter followers; LED circuits, supply-protection diodes and decoupling are omitted. The seven sliders are **assumed 10 kΩ linear**, because the cited sources give no value or taper. R/C values follow the Rev 1 BOM, including unpopulated capacitors. Op-amps approximate NE5532 with a single pole, gain 100,000, GBW 10 MHz (typical figures per [TI datasheet](https://www.ti.com/lit/gpn/NE5532)); no clipping, slew, noise or output-current limits are modeled. Ideal input source, 1 MΩ load. The DSP uses 0.5 V peak per full scale, not 5 V. A swept single-sine ngspice AC check with all faders at maximum yielded the highest op-amp output of 1.552 V peak at 0.5 V peak input, versus 15.513 V if mapped to 5 V peak. This proves only the swept-sine condition, not a bound for arbitrary broadband signals or actual output swing; real NE5532 stages clip before ideal rails.

## AC response and comparison

With one slider varied and all others centered, ngspice AC analysis relative to flat gives: 58.1 Hz ±6.83 dB, 154.6 Hz ±6.33 dB, 402.2 Hz ±6.18 dB, 987.5 Hz ±6.10 dB, 2.539 kHz ±6.12 dB, 6.452 kHz ±6.11 dB, and approximately 17.4 kHz ±6.48 dB. The physical labels are 63, 160, 410, 1k, 2.5k, 7.7k, 16k Hz. These model values are **not verified** against the actual slider specification. A secondary source claims ±15 dB; the modeled range is far smaller, so the unverified slider assumption is significant. The maker's page lists the Douglas Self reference frequencies but does not specify a gain range. `measure_ac.py` reproduces the calculation.

## Tests

A full validation passes at all 5 settings (2026-11-17; worst frequency response 0.090 dB). A later benchmark with nothing else validating measured 25.9% of one core. The circuit is linear: no modeled harmonics or compression, but validation of linear frequency/phase accuracy is still necessary. CPU measured 81.7% of one core per stereo instance at 48 kHz (machine load may affect this). Use DAW plugin rescan after installation.
