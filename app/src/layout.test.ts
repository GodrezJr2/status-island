import { expect, it } from 'vitest';
import { islandSize } from './layout';

it('uses approved island sizes and opacities', () => {
  expect(islandSize('idle', 0)).toMatchObject({ w: 132, h: 30, opacity: 0.63 });
  expect(islandSize('compact', 1)).toMatchObject({ w: 252, h: 34, opacity: 0.95 });
  expect(islandSize('stacked', 2)).toMatchObject({ w: 252, h: 66 });
  expect(islandSize('stacked', 5)).toMatchObject({ w: 252, h: 138 });
  expect(islandSize('hover', 1)).toMatchObject({ opacity: 0.36 });
  expect(islandSize('dropdown', 1)).toMatchObject({ w: 400, opacity: 1 });
});

it('uses the Figma radii and alert size', () => {
  expect(islandSize('compact', 1).radius).toBe(17);
  expect(islandSize('stacked', 2).radius).toBe(22);
  expect(islandSize('alert', 1)).toMatchObject({ w: 320, h: 34, radius: 17, opacity: 1 });
});
