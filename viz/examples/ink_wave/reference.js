// Render this scene with the upstream Ink Studio CLI.
const figure = require('./figure.json');

module.exports = ({ makeBuilder, makeSculpt, KIT }) => ({
  id: 'ink-wave',
  name: 'Shoulder wave',
  seed: figure.seed,
  build(seed) {
    const palette = figure.palette.map(([r, g, b]) => (r << 16) | (g << 8) | b);
    const builder = makeBuilder(seed, palette);
    const body = KIT.sculpt(makeSculpt(builder));
    for (const part of figure.fixed) {
      body.box(part.center, part.half, { r: part.ramp }, { res: figure.spacing });
    }
    body.bake();

    const arm = KIT.sculpt(makeSculpt(builder));
    const rig = KIT.rig(builder, arm);
    arm.box(figure.arm.center, figure.arm.half, { r: figure.arm.ramp }, { res: figure.spacing, tag: 'arm' });
    arm.bake();

    const world = builder.finish({ cam: figure.camera, dither: 0.25 });
    const stage = KIT.stage(world);
    rig.bind(stage);
    stage.add({ pre(_dt, time) {
      const angle = 0.43 * Math.PI * (1 - Math.cos(Math.PI * time));
      rig.set('arm', KIT.M.rot([0, 0, 1], angle), figure.shoulder, figure.shoulder);
      rig.apply();
    } });
    return world;
  }
});
