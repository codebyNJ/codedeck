#!/bin/zsh
# Builds target/release/Codedeck.app (release binary, Info.plist, mascot icon).
# ponytail: ad-hoc signed only; Developer ID signing + notarizing + .dmg come with distribution.
set -euo pipefail
cd "${0:A:h}/.."
cargo build --release
app=target/release/Codedeck.app
version=$(sed -n 's/^version = "\(.*\)"/\1/p' crates/codedeck/Cargo.toml | head -1)

# Icon: regenerate the .icns only when the mascot or the generator changed.
icns=packaging/AppIcon.icns
if [[ ! -f $icns || design/assets/mascot/codedeck-mascot.svg -nt $icns || packaging/icon.swift -nt $icns ]]; then
  tmp=$(mktemp -d)
  swift packaging/icon.swift design/assets/mascot/codedeck-mascot.svg $tmp/1024.png
  mkdir $tmp/AppIcon.iconset
  for s in 16 32 128 256 512; do
    sips -z $s $s $tmp/1024.png --out $tmp/AppIcon.iconset/icon_${s}x${s}.png >/dev/null
    sips -z $((s * 2)) $((s * 2)) $tmp/1024.png --out $tmp/AppIcon.iconset/icon_${s}x${s}@2x.png >/dev/null
  done
  iconutil -c icns $tmp/AppIcon.iconset -o $icns
  rm -rf $tmp
fi

rm -rf $app
mkdir -p $app/Contents/MacOS $app/Contents/Resources
cp target/release/Codedeck $app/Contents/MacOS/
cp $icns $app/Contents/Resources/
sed "s/VERSION/$version/g" packaging/Info.plist > $app/Contents/Info.plist
codesign --force --sign - $app
echo $app
