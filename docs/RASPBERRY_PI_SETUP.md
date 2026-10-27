# 🍓 LaserBurn Bridge & Web Server – Raspberry Pi & iPad Setup

LaserBurn Bridge muuttaa Raspberry Pi:n (esim. Raspberry Pi Zero 2 W, Pi 3, Pi 4 tai Pi 5) langattomaksi laser-palvelimeksi verstaalle.

Voit ohjata laseria, ajaa tiedostoja, siirtää akseleita ja valvoa työtä millä tahansa **iPadilla, iPhonella, Android-laitteella tai tietokoneella** ilman, että kallista tietokonetta tarvitsee pitää pölyisellä verstaspöydällä!

---

## 🚀 Pika-asennus Raspberry Pi:lle (1 komento)

Kytke Raspberry Pi laserisi USB-porttiin, kirjaudu sisään SSH:lla ja aja:

```bash
curl -sSL https://raw.githubusercontent.com/pietini69/laserburn/master/scripts/install_pi_bridge.sh | bash
```

Tämä skripti:
1. Asentaa kevyen `laserburn-bridge` -binäärin.
2. Lisää käyttäjäsi `dialout`-ryhmään USB-sarjaliikennettä varten.
3. Luo ja käynnistää automaattisen taustapalvelun (`systemd`), joka käynnistyy aina automaattisesti kun Raspi saa virtaa.

---

## 📱 iPad- ja iPhone-käyttö (Progressive Web App - PWA)

1. Yhdistä iPad samaan Wi-Fi -verkkoon kuin Raspberry Pi.
2. Avaa **Safari** ja siirry osoitteeseen:
   ```text
   http://laserburn.local:8080
   (tai http://<raspin-ip>:8080)
   ```
3. **Lisää kotivalikkoon:**
   - Napauta Safarin jakopainiketta (neliö ja nuoli ylös).
   - Valitse **"Lisää Koti-valikkoon"** (*Add to Home Screen*).
4. 🎉 LaserBurn aukeaa nyt omalla ikonillaan suoraan iPadin kotivalikosta **täyden ruudun itsenäisenä sovelluksena** ilman selaimen painikkeita tai osoitepalkkeja!

---

## ✨ Web Remoten ominaisuudet

- **Kosketusoptimoitu D-Pad & Jog:** Isot kosketuspainikkeet akseleille (X, Y, diagonaalit) ja askeleen pituuden valinta (0.1 mm, 1 mm, 5 mm, 10 mm, 50 mm, 100 mm).
- **Tuumat ja millimetrit rinnakkain:** 1-napin vaihto tuumien ja millimetrien välillä.
- **Reaaliaikaiset koordinaatit & 10Hz telemetria:** Näet pään liikkeet ja koneen tilan (Idle, Run, Alarm, Hold) suorana lähetyksenä.
- **Interaktiivinen 2D Työpöytä:** HTML5 Canvas, jossa näkyvät työpöydän mitat, ladatut leikkausvektorit ja reaaliaikainen laserpään sijainti. Zoomaa nipistämällä ja siirrä vetämällä.
- **Tiedostojen lataus & ajo:** Vedä ja pudota `.gcode`, `.nc`, `.lbrn2` (LightBurn) tai `.svg` suoraan selaimeen.
- **Turvallisuus & E-Stop:** Iso punainen **HÄTÄSEIS**-painike aina näkyvillä näytön yläkulmassa.
- **Tähtäysvalo & Framing:** Kytke 1% heikko tähtäysvalo päälle ja aja materiaalin rajaus (*Bounding Box Frame*).

---

## 🔌 Yhteensopivuus

- **Laserit:** Kaikki GRBL 1.1+ -pohjaiset laserit (Ortur, xTool, Sculpfun, Atomstack, Creality, Two Trees, Neje, Makerbase MKS DLC32 jne.) sekä virtuaalinen offline-simulaattoritila testausta varten.
- **Raspberry Pi -mallit:** Raspberry Pi Zero 2 W, Pi 3B+, Pi 4, Pi 5 ja mikä tahansa Debian/Ubuntu Linux ARM/ARM64 -laite.
- **Verkkoyhteys:** Toimii paikallisessa Wi-Fi -verkossa täysin ilman internet-yhteyttä.
