#!/bin/bash
# Create a simple 1024x1024 PNG icon
cat > /tmp/icon.svg << 'EOF'
<svg width="1024" height="1024" xmlns="http://www.w3.org/2000/svg">
  <rect width="1024" height="1024" fill="#667eea"/>
  <rect x="100" y="700" width="700" height="8" fill="white"/>
  <rect x="100" y="100" width="8" height="600" fill="white"/>
  <polyline points="120,600 250,400 400,500 600,200 750,300"
            stroke="#91cc75" stroke-width="12" fill="none"/>
  <circle cx="120" cy="600" r="20" fill="white"/>
  <circle cx="250" cy="400" r="20" fill="white"/>
  <circle cx="400" cy="500" r="20" fill="white"/>
  <circle cx="600" cy="200" r="20" fill="white"/>
  <circle cx="750" cy="300" r="20" fill="white"/>
</svg>
EOF

# Convert SVG to PNG using native tools if available
if command -v rsvg-convert &> /dev/null; then
    rsvg-convert -w 1024 -h 1024 /tmp/icon.svg -o source-icon.png
    echo "Icon created with rsvg-convert"
elif command -v convert &> /dev/null; then
    convert -background none -density 300 /tmp/icon.svg -resize 1024x1024 source-icon.png
    echo "Icon created with ImageMagick"
elif command -v qlmanage &> /dev/null; then
    # Use macOS Quick Look to render SVG
    qlmanage -t -s 1024 -o /tmp /tmp/icon.svg 2>/dev/null
    mv /tmp/icon.svg.png source-icon.png 2>/dev/null || echo "qlmanage failed"
else
    echo "No SVG converter found, please install imagemagick or librsvg"
    exit 1
fi
