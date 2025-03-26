mod decryptor;
use decryptor::Decryptor;
const DEFAULT_SEGMENT_SIZE: usize = 0x1400;
fn main() {
    let mut decryptor = Decryptor::new(
        "../input",
        "../output",
        DEFAULT_SEGMENT_SIZE,
        "mCwRNjKcGH5TFYGRu7dq49z4jsqefM0Gb5Z0xjl87mkQvCKxr18RRXqC64G1cOm32387THyBX8Fh727f2g3H7D1YME6gM04mmjY7HbjJ3uEy6VGszDSzL39uokoLmKp7F3BETIPLRy1fStV69C0uXD3UpJE1hHDE5qPu8Er35I6x31qsw9WbeJ37oby7787vPeJ6om5wIR6jQIec3I7l8y4LWgr5puV8Ea9Y6E12w9waNgsPU9dAP2XF9k26gT6aajoY1sFkWYga5Oe1l6ph14YNDn7Al871HwwgeiMbLb0HvgvzNfBJ8LRw5pDcp1ntcNjkUi53GzE6NI1eB4t5n8EP458Zakn4J3uUS336n2PoBP307G8w7SCWKwC22754H8121xBzHoimr0fHNwPoADi32ZQ5v03nv0mgyssZUfo0T6ETLm4264Tvp26S5K2xWOHg01J7r0ktHx2E113h9loDr1MGpi6LIes1nM6jd1IK5A023ySr3M9Kupn49lq2").unwrap();
    decryptor.decrypt().unwrap();
}
