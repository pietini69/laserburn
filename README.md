# LaserBurn 🦀⚡

Moderni, huippunopea ja kevyt avoimen lähdekoodin laserleikkaus- ja kaiverrus-CAM-ohjelmisto (Rust). 

Tämä ohjelmisto kehitettiin takaisinmallintamalla (Reverse Engineering) teollisuusstandardi **LightBurn (v2.0.02)** käyttäen REA- ja Mach-O -analyysityökaluja. Se tarjoaa suoran `.lbrn2`-projektitiedostojen yhteensopivuuden, edistyneen CAM-radanoptimoinnin, kuvan rasterikaiverrus-ditheröinnin sekä tuki useille laserohjaimille (GRBL, Smoothie, Marlin, Ruida DSP).

---

## 🎯 Pääominaisuudet

### 1. Täysi LightBurn `.lbrn2` & `.svg` -yhteensopivuus
- Lukee ja kirjoittaa LightBurnin natiivia `.lbrn2` JSON -tiedostomuotoa.
- Tukee monitasoisia leikkauskerroksia (C00–C29) omine nopeus-, teho-, toisto- ja ilmansyöttöasetuksineen.
- Tukee standardia SVG-vektoritiedostojen tuontia ja vientiä.

### 2. Älykäs leikkausradan optimointi (`CutOptimizer`)
- **Inner-cuts-first:** Tunnistaa sisäkkäiset geometriat (reiät, kirjaimet, sisämuodot) ja leikkaa ne AINA ennen ulkoreunaa, estäen kappaleen putoamisen tai liikkumisen kesken leikkauksen.
- **Automaattinen Kerf-kompensaatio (Säteen leveyden korjaus):**
  - Huomioi lasersäteen polttojäljen paksuuden (beam kerf, esim. 0.10 mm) ja kompensoi leikkausradat automaattisesti:
    - Ulkomuodot laajennetaan ulospäin $+kerf / 2$ (+0.05 mm)
    - Sisäaukot ja kolot pienennetään sisäänpäin $-kerf / 2$ (-0.05 mm)
  - Sormiliitokset, laatikot ja palapelit istuvat täydellisen napakasti suoraan leikkauksesta.
  - **Määritelmä:** LaserBurniin syötetään säteen **koko polttoleveys** (esim. `0.100 mm`). Ohjelma laskee ja näyttää suoraan käyttöliittymässä todellisen työstöradan siirtymän (`±0.050 mm`). *(LightBurnissa kentän nimi oli 'Kerf Offset', johon käyttäjän piti laskea puolikas itse).*
- **TSP (Traveling Salesperson) matkaoptimointi:** Minimoi laser sammutettuna tapahtuvat pikasiirrot (rapid travel) lähimmän naapurin algoritmilla.
- **Suuntakääntö (Direction reversal):** Kääntää avoimet viivat tarvittaessa optimoidakseen työkalun liikkeen.
- **Sillat ja kiinnikkeet (Tabs & Bridges):** Mahdollisuus jättää kappaleeseen ohuita mikrokiinnikkeitä, jotta irtoavat osat pysyvät kiinni materiaalilevyssä.

### 3. Vektoritäyttö ja taskutus (`Hatch Engine`)
- Automaattinen rasteritaskutus (Fill mode) mielivaltaisella skannauskulmalla (0°–360°) ja viivavälillä (esim. 0.08 mm = ~318 DPI).
- Jatkuva serpentiinirata (zigzag) vierekkäisten viivojen välillä ilman turhia lasersammutuksia.
- Ristiintäyttö (Cross-hatch) kahdessa toisiaan vastaan kohtisuorassa kulmassa.

### 4. Teollisuustason kuvan ditheröinti (8 algoritmia)
Ultra-nopea monisäikeinen rasterikuvien ditheröinti laserkaiverrusta varten:
1. **Atkinson** (Macintosh-klassikko, tuottaa erittäin puhtaat ja terävät rasteripisteet ilman suttua)
2. **Floyd-Steinberg** (Klassinen virheenlevitys)
3. **Jarvis-Judice-Ninke** (Korkea dynamiikka)
4. **Stucki** (Pehmeä ja tarkka sävyala)
5. **Ordered Bayer 8x8** (Säännöllinen matriisiditheröinti)
6. **Halftone** (Sanomalehti / pyöreät rasteripisteet)
7. **Threshold & SmartThreshold** (Mustavalko-rajaus)
8. **Grayscale Power Modulation** (3D-relief / tehomoduloitu kaiverrus)

### 5. Moniohjaintuki
- **GRBL / Smoothie / Marlin (G-Code):**
  - Dynaaminen laserteho (`M4` GRBL 1.1f+ -tilassa estää palamisen nurkissa).
  - Vakio laserteho (`M3`).
  - Automaattinen ilmansyöttö (`M8` päälle / `M9` pois).
  - Kappaleen kohdistuskehys (`Framing` matalatehoisella lasertähdäyksellä).
- **Ruida DSP Controllers (RDC6442G, RDC6445G jne.):**
  - Suora binäärinen `.rd`-työtiedostojen generointi.
  - UDP-verkkoprotokolla portissa 50200.

### 6. Leikkaussimulaattori ja aika-arvio (`Simulator`)
- Laskee todellisen leikkausmatkan, pikasiirtomatkan sekä laserin käyttöasteen (duty cycle).
- Huomioi koneen kiihtyvyyden ($mm/s^2$) ja maksiminopeuden puolisuunnikas- (trapezoidal) ja kolmioprofiililla.

---

## 🚀 Asennus ja kääntäminen

Rust on asennettuna ja ohjelma kääntyy suoraan:

```bash
cd /Volumes/Dataa/Ohjelmointi/ProjektitRust/laser_burn

# Käännä julkaisuversio (CLI)
cargo build --release --no-default-features

# Käännä graafisella käyttöliittymällä (GUI)
cargo build --release
```

---

## 🛠️ Käyttöesimerkkejä CLI:stä

### 1. Tarkastele LightBurn `.lbrn2`- tai `.svg`-projektia
```bash
./target/release/laser_burn info gear_demo.lbrn2
```

### 2. Generoi optimoitu GRBL G-koodi
```bash
./target/release/laser_burn gcode gear_demo.lbrn2 -o leikkaus.gcode --rapid 6000 --max-s 1000
```

### 3. Generoi Ruida DSP `.rd` -binääritiedosto
```bash
./target/release/laser_burn ruida gear_demo.lbrn2 -o tyotiedosto.rd
```

### 4. Simuloi leikkausaika
```bash
./target/release/laser_burn simulate gear_demo.lbrn2 --accel 3000
```

### 5. Ditheroi valokuva laserkaiverrusta varten
```bash
./target/release/laser_burn dither kuva.png -o dither_atkinson.png --algorithm atkinson
```

### 6. Luo testiprojekti (hammasratas kiinnitysreikineen)
```bash
./target/release/laser_burn demo --output uusi_ratas.lbrn2
```
