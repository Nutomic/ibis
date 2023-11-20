#[test]
fn test_diff() {
    use diffy::create_patch;

    let original = "The Way of Kings\nWords of Radiance\n";
    let modified = "The Way of Kings\nWords of Radiance\nOathbringer\n";

    let patch = create_patch(original, modified);
    assert_eq!("--- original\n+++ modified\n@@ -1,2 +1,3 @@\n The Way of Kings\n Words of Radiance\n+Oathbringer\n", patch.to_string());
}
