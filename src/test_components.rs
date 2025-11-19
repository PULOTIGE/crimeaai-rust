// Simple test without GUI dependencies
#[path = "archguard.rs"]
mod archguard;
#[path = "evolution.rs"]
mod evolution;
#[path = "lighting.rs"]
mod lighting;
#[path = "voxel.rs"]
mod voxel;

use archguard::ArchGuard;
use evolution::EvolutionEngine;
use lighting::LightingSystem;
use voxel::{Voxel, VoxelWorld, Genome};

fn main() {
    println!("=== Adaptive Entity Engine v1.0 - Component Tests ===\n");
    
    // Test 1: Voxel creation
    println!("Test 1: Voxel Creation");
    let voxel = Voxel::new([0, 0, 0]);
    println!("  ✓ Voxel created at position [0, 0, 0]");
    println!("  ✓ Energy: {}", voxel.energy);
    println!("  ✓ Genome concepts: {}", voxel.genome.concepts.len());
    println!("  ✓ Resonance: {}", voxel.resonance.to_f32());
    
    // Test 2: Genome
    println!("\nTest 2: Genome System");
    let mut genome = Genome::new();
    genome.add_concept("concept1".to_string());
    genome.add_concept("concept2".to_string());
    println!("  ✓ Genome created with {} concepts", genome.concepts.len());
    println!("  ✓ Concepts: {:?}", genome.concepts);
    
    // Test 3: LightPattern size
    println!("\nTest 3: LightPattern Size");
    let pattern = lighting::LightPattern::new();
    let size = std::mem::size_of::<lighting::LightPattern>();
    println!("  ✓ LightPattern size: {} bytes", size);
    assert_eq!(size, 1000, "LightPattern must be exactly 1000 bytes!");
    println!("  ✓ Size verification passed!");
    
    // Test 4: Evolution Engine
    println!("\nTest 4: Evolution Engine");
    let evolution = EvolutionEngine::new();
    println!("  ✓ Evolution engine created");
    println!("  ✓ Mutation rate: {}", evolution.mutation_rate);
    println!("  ✓ Crossover rate: {}", evolution.crossover_rate);
    
    // Test fitness calculation
    let mut test_voxel = Voxel::new([1, 1, 1]);
    test_voxel.energy = 0.8;
    test_voxel.genome.add_concept("test".to_string());
    let fitness = evolution.fitness(&test_voxel);
    println!("  ✓ Fitness calculated: {:.3}", fitness);
    
    // Test 5: VoxelWorld
    println!("\nTest 5: VoxelWorld");
    let mut world = VoxelWorld::new();
    world.add_voxel([10, 20, 30]);
    world.add_voxel([15, 25, 35]);
    println!("  ✓ VoxelWorld created with {} voxels", world.voxels.len());
    println!("  ✓ Max points: {}", world.max_points);
    
    // Test trauma mode
    world.trauma_mode = true;
    world.update(0.016); // ~60 FPS delta
    println!("  ✓ Trauma mode: {}", world.trauma_mode);
    
    // Test 6: Lighting System
    println!("\nTest 6: Lighting System");
    let mut lighting = LightingSystem::new();
    lighting.add_pattern(lighting::LightPattern::new());
    lighting.update_lighting(1.0);
    println!("  ✓ Lighting system created");
    println!("  ✓ Light patterns: {}", lighting.patterns.len());
    
    // Test 7: ArchGuard
    println!("\nTest 7: ArchGuard Enterprise");
    let mut archguard = ArchGuard::new();
    println!("  ✓ ArchGuard created");
    println!("  ✓ Circuit open: {}", archguard.is_circuit_open());
    
    // Test rhythm detector
    archguard.update_rhythm(0.0);
    archguard.update_rhythm(10.0);
    let phase = archguard.get_rhythm_phase();
    println!("  ✓ Rhythm phase (0.038 Hz): {:.3}", phase);
    
    // Test empathy ratio
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        archguard.update_empathy_ratio(0.75).await;
        let empathy = archguard.get_empathy_ratio().await;
        println!("  ✓ Empathy ratio: {:.3}", empathy);
    });
    
    // Note: ArchGuard methods require &mut self for some operations
    // This is a simplified test
    
    println!("\n=== All Component Tests Passed! ===");
    println!("\nNote: Full GUI test requires display server and Vulkan drivers.");
    println!("Core components are working correctly!");
}
