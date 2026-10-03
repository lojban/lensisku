fn main() {
    // Refinery's macro tracks existing migration files through include_str!,
    // but Cargo must also notice migrations added to or removed from the folder.
    println!("cargo:rerun-if-changed=migrations");
}
