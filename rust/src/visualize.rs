use std::fs;
#[cfg(target_os = "macos")]
use std::process::Command;

use slug::slugify;

use crate::Stop;

pub fn visualize_distance(stop_a: &Stop, stop_b: &Stop) -> std::io::Result<()> {
    let html = format!(r#"<!DOCTYPE html>
<html>
<head>
    <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/leaflet/1.9.4/leaflet.min.css" />
    <style>body {{ margin: 0; }} #map {{ height: 100vh; }}</style>
</head>
<body>
    <div id="map"></div>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/leaflet/1.9.4/leaflet.min.js"></script>
    <script>
        const map = L.map('map');
        L.tileLayer('https://{{s}}.tile.openstreetmap.org/{{z}}/{{x}}/{{y}}.png').addTo(map);
        
        L.marker([{}, {}]).addTo(map).bindPopup('{}');
        L.marker([{}, {}]).addTo(map).bindPopup('{}');
        L.polyline([[{}, {}], [{}, {}]], {{color: 'red'}}).addTo(map);
        
        map.fitBounds([[{}, {}], [{}, {}]]);

        map.setZoom(30);

    </script>
</body>
</html>"#, 
        stop_a.lat, stop_a.long, stop_a.name,
        stop_b.lat, stop_b.long, stop_b.name, 
        stop_a.lat, stop_a.long, stop_b.lat, stop_b.long,
        stop_a.lat, stop_a.long, stop_b.lat, stop_b.long
    );

    let name = format!("maps/{}-{}.html", slugify(&stop_a.name), slugify(&stop_b.name));
    
    fs::write(&name, html).unwrap();
    
    // Open it
    #[cfg(target_os = "macos")]
    Command::new("open").arg(name).spawn()?;
    
    Ok(())
}