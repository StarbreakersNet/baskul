// Bruit simplex 3D (Stefan Gustavson), dans [-1, 1] : la même graine donne le même fond à chaque ouverture.
// Appelé des dizaines de milliers de fois par image : aucune allocation dans noise3

const GRADIENTS = new Float64Array([
  1, 1, 0, -1, 1, 0, 1, -1, 0, -1, -1, 0, 1, 0, 1, -1, 0, 1, 1, 0, -1, -1, 0, -1, 0, 1, 1, 0, -1, 1,
  0, 1, -1, 0, -1, -1,
]);
const F3 = 1 / 3;
const G3 = 1 / 6;
const SEED = 1337;

const PERMUTATION = buildPermutation(SEED);

function buildPermutation(seed: number) {
  const values = Array.from({ length: 256 }, (_, index) => index);
  let state = seed;
  for (let i = 255; i > 0; i--) {
    state = (state * 16807) % 2147483647;
    const j = Math.floor((state / 2147483647) * (i + 1));
    [values[i], values[j]] = [values[j], values[i]];
  }
  const table = new Uint8Array(512);
  for (let i = 0; i < 512; i++) {
    table[i] = values[i & 255];
  }
  return table;
}

function corner(x: number, y: number, z: number, hash: number) {
  const falloff = 0.6 - x * x - y * y - z * z;
  const g = (hash % 12) * 3;
  return falloff < 0
    ? 0
    : falloff ** 4 * (GRADIENTS[g] * x + GRADIENTS[g + 1] * y + GRADIENTS[g + 2] * z);
}

export function noise3(xin: number, yin: number, zin: number) {
  const skew = (xin + yin + zin) * F3;
  const i = Math.floor(xin + skew);
  const j = Math.floor(yin + skew);
  const k = Math.floor(zin + skew);
  const unskew = (i + j + k) * G3;
  const x0 = xin - (i - unskew);
  const y0 = yin - (j - unskew);
  const z0 = zin - (k - unskew);
  // Deuxième et troisième sommets du tétraèdre qui contient le point
  const i1 = x0 >= y0 && x0 >= z0 ? 1 : 0;
  const j1 = y0 > x0 && y0 >= z0 ? 1 : 0;
  const k1 = 1 - i1 - j1;
  const i2 = x0 >= y0 || x0 >= z0 ? 1 : 0;
  const j2 = y0 > x0 || y0 >= z0 ? 1 : 0;
  const k2 = z0 > x0 || z0 > y0 ? 1 : 0;
  const p = PERMUTATION;
  const ii = i & 255;
  const jj = j & 255;
  const kk = k & 255;
  return (
    32 *
    (corner(x0, y0, z0, p[ii + p[jj + p[kk]]]) +
      corner(x0 - i1 + G3, y0 - j1 + G3, z0 - k1 + G3, p[ii + i1 + p[jj + j1 + p[kk + k1]]]) +
      corner(
        x0 - i2 + 2 * G3,
        y0 - j2 + 2 * G3,
        z0 - k2 + 2 * G3,
        p[ii + i2 + p[jj + j2 + p[kk + k2]]]
      ) +
      corner(x0 - 0.5, y0 - 0.5, z0 - 0.5, p[ii + 1 + p[jj + 1 + p[kk + 1]]]))
  );
}
