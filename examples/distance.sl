square: {dup *}

sqrt: {0.5 **}

// x1 y1 x2 y2 -> distance
distance: {
  rot
  - square
  rot rot
  - square
  +
  sqrt
}

3 0      // first point
0 4      // second point
distance // 5
