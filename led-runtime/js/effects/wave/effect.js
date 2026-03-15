// Auto-generated – do not edit. Run: just compile-effects
// Target: mquickjs (ES6, no modules)

// ── effect-runtime ──
/**
 * Shared LED effect utilities.
 *
 * Color types (tagged):
 *   { mode: 'rgb', r, g, b }   – 0-255 per channel
 *   { mode: 'hsv', h, s, v }   – h: 0-360, s/v: 0-1
 *
 * leds buffer: Uint8Array of length ledCount*3, layout [R,G,B, R,G,B, ...]
 */
/** Convert HSV (h 0-360, s/v 0-1) to [r, g, b] tuple. */
function hsvToRgb(h, s, v) {
    h = ((h % 360) + 360) % 360;
    const c = v * s;
    const x = c * (1 - Math.abs(((h / 60) % 2) - 1));
    const m = v - c;
    let r = 0, g = 0, b = 0;
    if (h < 60) {
        r = c;
        g = x;
    }
    else if (h < 120) {
        r = x;
        g = c;
    }
    else if (h < 180) {
        g = c;
        b = x;
    }
    else if (h < 240) {
        g = x;
        b = c;
    }
    else if (h < 300) {
        r = x;
        b = c;
    }
    else {
        r = c;
        b = x;
    }
    return [Math.round((r + m) * 255), Math.round((g + m) * 255), Math.round((b + m) * 255)];
}
/** Convert a tagged color ({ mode:'rgb' } or { mode:'hsv' }) to [r, g, b] tuple. */
function toRgb(c) {
    if (c.mode === 'hsv')
        return hsvToRgb(c.h, c.s, c.v);
    return [c.r, c.g, c.b];
}
/** Create a zeroed Uint8Array for ledCount LEDs (3 bytes each). */
function makeBlank(ledCount) {
    return new Uint8Array(ledCount * 3);
}
/**
 * Create a base LED state machine with common lifecycle methods.
 */
function createBaseMachine(ledCount, speed, handlers) {
    const machine = {
        status: 'idle',
        speed: speed,
        ledCount: ledCount,
        leds: makeBlank(ledCount),
        tick: function () {
            if (machine.status !== 'running')
                return;
            handlers.tick(machine);
        },
        start: function () {
            if (machine.status === 'idle')
                machine.status = 'running';
        },
        pause: function () {
            if (machine.status === 'running')
                machine.status = 'paused';
        },
        resume: function () {
            if (machine.status === 'paused')
                machine.status = 'running';
        },
        stop: function () {
            machine.status = 'idle';
            if (handlers.reset)
                handlers.reset();
            machine.leds = makeBlank(ledCount);
        },
        setSpeed: function (ms) { machine.speed = ms; },
        setConfig: function (key, value) {
            if (key === 'speed')
                machine.speed = value;
            else if (handlers.setConfig)
                handlers.setConfig(key, value);
        }
    };
    return machine;
}

// ── effect: wave ──
function createEffect(config) {
    const cfg = config || {};
    const ledCount = cfg.ledCount || 20;
    let color = toRgb(cfg.color || { mode: 'rgb', r: 52, g: 211, b: 153 });
    let waveWidth = cfg.waveWidth || 5;
    let position = 0;
    return createBaseMachine(ledCount, cfg.speed || 100, {
        tick: function (m) {
            const buf = m.leds;
            buf.fill(0);
            for (let i = 0; i < waveWidth; i++) {
                const o = ((position + i) % ledCount) * 3;
                buf[o] = color[0];
                buf[o + 1] = color[1];
                buf[o + 2] = color[2];
            }
            position = (position + 1) % ledCount;
        },
        reset: function () { position = 0; },
        setConfig: function (key, value) {
            if (key === 'color')
                color = toRgb(value);
            if (key === 'waveWidth')
                waveWidth = value;
        }
    });
}
