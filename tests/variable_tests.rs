use rustplex::Var;
use std::collections::HashSet;

/// Test: Variable Identity
/// Logic: Two variables created separately are NOT equal, even if they share a name.
#[test]
fn test_variable_uniqueness() {
    let x1 = Var::new().with_name("x");
    let x2 = Var::new().with_name("x"); // Different instance

    assert_ne!(x1, x2, "Two distinct variables should not be equal");
}

/// Test: Variable Cloning
/// Logic: A cloned variable points to the EXACT same data (Reference Counting).
#[test]
fn test_variable_cloning_identity() {
    let x1 = Var::new().with_name("x");
    let x1_clone = x1.clone();

    assert_eq!(x1, x1_clone, "A clone should be equal to the original");
}

/// Test: Hashing Consistency
/// Logic: If x1 == x1_clone, they must produce the same hash (for HashMap keys).
/// x1 != x2, so they should ideally produce different hashes (though collisions are possible, they shouldn't define equality).
#[test]
fn test_variable_hashing() {
    let x1 = Var::new().with_name("x");
    let x2 = Var::new().with_name("y");
    let x1_clone = x1.clone();

    let mut set = HashSet::new();
    set.insert(x1.clone());
    
    // x1_clone should already be in the set
    assert!(set.contains(&x1_clone));
    
    // x2 should NOT be in the set
    assert!(!set.contains(&x2));
}

/// Test: Interior Mutability
/// Logic: Modifying a clone should modify the original (shared state).
#[test]
fn test_interior_mutability() {
    let x = Var::new();
    let x_clone = x.clone();
    
    // Modify 'x'
    x.with_lower_bound(10.0);
    
    // 'x_clone' should reflect this change because they share the same RefCell
    assert_eq!(x_clone.lower_bound(), 10.0);
}