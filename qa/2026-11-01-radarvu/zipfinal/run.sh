cd "/Volumes/Developer/OpenGrid/RadarVU/qa/2026-11-01-radarvu"
auval -v aufx RdVU OpGr > zipfinal/auval.txt 2>&1
/Users/marcosdeida/.agent/skills/plugin-daw-test/scripts/aax_validate.sh "${UNZIPPED:-unz}/RadarVU.aaxplugin" "/Volumes/Developer/OpenGrid/RadarVU/qa/2026-11-01-radarvu/zipfinal/aax" > zipfinal/aax-validate.txt 2>&1
python3 /Users/marcosdeida/.agent/skills/plugin-daw-test/scripts/reaper_qa.py launch --qa-dir /Volumes/Developer/OpenGrid/RadarVU/qa/2026-11-01-radarvu/zipfinal/r-vst3 > zipfinal/l1.txt 2>&1
python3 /Users/marcosdeida/.agent/skills/plugin-daw-test/scripts/reaper_qa.py suite --qa-dir /Volumes/Developer/OpenGrid/RadarVU/qa/2026-11-01-radarvu/zipfinal/r-vst3 --fx "VST3: RadarVU (OpenGrid)" --exceptions exceptions.txt > zipfinal/suite-vst3.txt 2>&1
python3 /Users/marcosdeida/.agent/skills/plugin-daw-test/scripts/reaper_qa.py quit --qa-dir /Volumes/Developer/OpenGrid/RadarVU/qa/2026-11-01-radarvu/zipfinal/r-vst3 > /dev/null 2>&1; sleep 4
python3 /Users/marcosdeida/.agent/skills/plugin-daw-test/scripts/reaper_qa.py launch --qa-dir /Volumes/Developer/OpenGrid/RadarVU/qa/2026-11-01-radarvu/zipfinal/r-au > zipfinal/l2.txt 2>&1
python3 /Users/marcosdeida/.agent/skills/plugin-daw-test/scripts/reaper_qa.py suite --qa-dir /Volumes/Developer/OpenGrid/RadarVU/qa/2026-11-01-radarvu/zipfinal/r-au --fx "AU: RadarVU (OpenGrid)" --exceptions exceptions.txt > zipfinal/suite-au.txt 2>&1
python3 /Users/marcosdeida/.agent/skills/plugin-daw-test/scripts/reaper_qa.py quit --qa-dir /Volumes/Developer/OpenGrid/RadarVU/qa/2026-11-01-radarvu/zipfinal/r-au > /dev/null 2>&1
echo finished > zipfinal/done.txt
