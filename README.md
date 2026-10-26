# LaserBurn v1.0.0 🦀⚡

Moderni, huippunopea ja kevyt monialustainen (macOS, Windows, Linux) laserleikkaus- ja kaiverrus-CAM-ohjelmisto (Rust / `eframe` / `egui`).

Ohjelmisto tarjoaa suoran LightBurn `.lbrn2`- ja `.svg`-projektitiedostojen yhteensopivuuden, edistyneen CAM-radanoptimoinnin, todellisen analogisen harmaasävy-tehomodulaation (`GrayscalePower`), keramiikkalaattojen maalikerrossimulaattorin, kamerakohdistuksen automaattisella aihion tunnistuksella sekä suoran tuen useille laserohjaimille (GRBL 1.1+, Smoothie, Marlin, Ruida DSP).

---

## 🎯 Pääominaisuudet (v1.0.0)

### 1. Täysi LightBurn `.lbrn2` & `.svg` -yhteensopivuus
- Lukee ja kirjoittaa LightBurnin `.lbrn2`-projektitiedostoja.
- Tukee monitasoisia leikkauskerroksia (C00–C29) omine nopeus-, teho-, toisto-, ditheröinti- ja ilmansyöttöasetuksineen.
- Täysi SVG-vektoritiedostojen tuonti ja vienti 2D-affiinimatriisimuunnoksilla (`matrix`, `translate`, `scale`, `rotate`, `<use>`).

### 2. True Analog Grayscale Power Modulation & Keramiikkalaattojen Maalikerrossimulaattori
- **Grayscale Power Modulation (`M4 S...`):** Säätelee dynaamisesti laserin PWM-tehoa lineaarisesti pikselin tummuuden mukaan välillä `Min Power %` – `Max Power %`.
- **Overscan (Lead-in / Lead-out):** Kiihdyttää ja jarruttaa laserpään varsinaisen polttoalueen ulkopuolella 0 % teholla, jolloin laatan tai puun reunat eivät pala tummiksi.
- **Negative / Invert:** Kääntää harmaasävyt tummaksi maalatuille keramiikkalaatoille, joissa laser paljastaa alla olevan vaalean/värillisen kerroksen.
- **Ceramic Tile Paint Layer Simulator:** Simuloi ja esikatselee suoraan työalueella, miten eri lasertehot polttavat päällekkäisten spraymaalikerrosten läpi aina pohjalaattaan saakka.

### 3. Vektori-CAD, 2D Boolean -työkalut, SDF Contour Offset & Undo/Redo
- **Monivaiheinen Undo / Redo (`Cmd+Z` / `Cmd+Shift+Z`):** Täydellinen muokkaushistoria kaikille piirto-, siirto-, skaalaus- ja boolean-toiminnoille.
- **2D Boolean -operaatiot:** Yhdistä (`Weld / Union`), vähennä (`Subtract A - B`) ja leikkaa (`Intersect`) päällekkäisiä vektorimuotoja ja tekstiä.
- **True SDF Contour Offset:** Luo tarkat ulko- tai sisäpuoliset ääriviivasiirtymät (esim. tarrojen leikkausreunat tai korujen kehykset) Signed Distance Field -laskennalla.
- **Automaattinen Kerf-kompensaatio:** Huomioi säteen polttoleveyden (`kerf`) laajentaen ulkoreunoja $+kerf/2$ ja pienentäen sisäreikiä $-kerf/2$.

### 4. Sisäänrakennettu Materiaalin Testimatriisi (Speed × Power Grid Generator)
- Generoi yhdellä klikkauksella valmiin `Nopeus × Teho` -kalibrointiruudukon automaattisilla vektoriteksteillä (C00 tekstit, C01–C29 testiruudut) mille tahansa materiaalille.

### 5. Teollisuustason kuvan ditheröinti (9 tilaa)
1. **Atkinson** (Erittäin terävät ja puhtaat rasteripisteet)
2. **Floyd-Steinberg** (Klassinen virheenlevitys)
3. **Jarvis-Judice-Ninke** (Korkea dynamiikka)
4. **Stucki** (Pehmeä sävyala)
5. **Burkes & Sierra3** (Nopea ja tasainen virheenlevitys)
6. **Ordered Bayer 8x8** (Säännöllinen matriisiditheröinti)
7. **Halftone** (Sanomalehtirasteri)
8. **Threshold & SmartThreshold** (Mustavalko-rajaus)
9. **Grayscale Power Modulation** (Analoginen PWM-tehomodulaatio)

### 6. Kamerakohdistus & 1-Click Workpiece Auto-Fit
- **4 pisteen perspektiivikalibrointi (Homography $H$ ja $H^{-1}$)** sekä kelluva **2x–8x Zoom Loupe** tarkkaan kohdistukseen.
- **1-Click Detect & Center on Workpiece:** Tunnistaa kamerakuvasta alustalla olevan suorakaiteen muotoisen aihion (esim. keraaminen laatta tai vanerinpala) ja keskittää/sovittaa valitut kuviot automaattisesti sen päälle.

### 7. Reaaliaikainen Data Traffic Log (TX / RX) & Selkokielinen G-kooditulkki
- Oma rinnakkainen **Data Traffic (TX / RX)** -paneeli näyttää jokaisen lähetetyn G-koodin ja GRBL-vastauksen reaaliajassa.
- **`[✓] Explain`:** Kääntää jokaisen G-koodin, M-koodin, GRBL-asetuksen (`$`), tilaviestin (`<Idle|MPos:...>`) ja virheen/hälytyksen suoraan selkokieliseksi englanniksi.
- **`[✓] Save to Log File`:** Tallentaa kaiken sarjaliikenteen aikaleimoilla suoraan tiedostoon `~/Documents/LaserBurn_traffic.log`.

### 8. Tumma & Vaalea tila, Dynaaminen 0,0-Jog-ohjaus & macOS Apple Shortcuts
- **Theme: Dark / Light:** Korkean kontrastin tumma ja vaalea käyttöliittymäteema.
- **Origin-Aware Jog D-Pad:** `X+`/`X-`/`Y+`/`Y-`-jog-painikkeet mukautuvat automaattisesti sen mukaan, missä kulmassa koneen `0,0`-origo sijaitsee.
- **macOS Apple Shortcuts -automaatio (`#[cfg(target_os = "macos")]`):** Käynnistä valittu macOS-pikakomento automaattisesti, kun poltto alkaa (`On Job Start`) tai valmistuu (`On Job Complete`).

---

## 🚀 Kääntäminen ja jakelupaketit

```bash
cd /Volumes/Dataa/Ohjelmointi/ProjektitRust/laser_burn

# Suorita yksikkötestit:
cargo test --features gui

# Rakenna kaikki jakelupaketit (macOS Universal 2 DMG/ZIP, Windows x64 ZIP, Linux x86_64 tar.gz)
# ja päivitä /Applications/LaserBurn.app:
./scripts/build_all.sh
```
