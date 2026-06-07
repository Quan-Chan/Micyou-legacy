import { Hct, SchemeTonalSpot, argbFromHex, MaterialDynamicColors } from "@material/material-color-utilities";

function argbToHslString(argb) {
  const r = (argb >> 16) & 255;
  const g = (argb >> 8) & 255;
  const b = argb & 255;
  
  const rf = r / 255;
  const gf = g / 255;
  const bf = b / 255;
  const max = Math.max(rf, gf, bf), min = Math.min(rf, gf, bf);
  let h, s, l = (max + min) / 2;

  if(max == min){
      h = s = 0; // achromatic
  }else{
      var d = max - min;
      s = l > 0.5 ? d / (2 - max - min) : d / (max + min);
      switch(max){
          case rf: h = (gf - bf) / d + (gf < bf ? 6 : 0); break;
          case gf: h = (bf - rf) / d + 2; break;
          case bf: h = (rf - gf) / d + 4; break;
      }
      h /= 6;
  }
  return `${Math.round(h * 360)} ${Math.round(s * 100)}% ${Math.round(l * 100)}%`;
}

const hct = Hct.fromInt(argbFromHex("#4A672D"));
const schemeLight = new SchemeTonalSpot(hct, false, 0);
const schemeDark = new SchemeTonalSpot(hct, true, 0);

const colors = [
  'primary', 'onPrimary', 'primaryContainer', 'onPrimaryContainer',
  'secondary', 'onSecondary', 'secondaryContainer', 'onSecondaryContainer',
  'tertiary', 'onTertiary', 'tertiaryContainer', 'onTertiaryContainer',
  'error', 'onError', 'errorContainer', 'onErrorContainer',
  'background', 'onBackground', 'surface', 'onSurface',
  'surfaceVariant', 'onSurfaceVariant', 'outline', 'outlineVariant',
  'shadow', 'scrim', 'inverseSurface', 'inverseOnSurface', 'inversePrimary',
  'surfaceDim', 'surfaceBright', 'surfaceContainerLowest', 'surfaceContainerLow',
  'surfaceContainer', 'surfaceContainerHigh', 'surfaceContainerHighest'
];

const sLight = {};
const sDark = {};
for (const color of colors) {
    sLight[color] = MaterialDynamicColors[color].getArgb(schemeLight);
    sDark[color] = MaterialDynamicColors[color].getArgb(schemeDark);
}

console.log(":root {");
for(let key in sLight) {
  let kebab = key.replace(/([a-z0-9]|(?=[A-Z]))([A-Z])/g, '$1-$2').toLowerCase();
  console.log(`    --${kebab}: ${argbToHslString(sLight[key])};`);
}
console.log("}\n.dark {");
for(let key in sDark) {
  let kebab = key.replace(/([a-z0-9]|(?=[A-Z]))([A-Z])/g, '$1-$2').toLowerCase();
  console.log(`    --${kebab}: ${argbToHslString(sDark[key])};`);
}
console.log("}");
