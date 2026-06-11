export interface Engram {
  id: number
  name: string
  level_required: number
  points_cost: number
  category: string
}

export const ENGRAMS_DB: Engram[] = [
  // Weapons
  { id: 1, name: 'Stone Pickaxe', level_required: 1, points_cost: 3, category: 'Weapons' },
  { id: 2, name: 'Stone Hatchet', level_required: 1, points_cost: 3, category: 'Weapons' },
  { id: 3, name: 'Slingshot', level_required: 5, points_cost: 5, category: 'Weapons' },
  { id: 4, name: 'Spear', level_required: 10, points_cost: 6, category: 'Weapons' },
  { id: 5, name: 'Bow', level_required: 15, points_cost: 10, category: 'Weapons' },
  { id: 6, name: 'Stone Sword', level_required: 12, points_cost: 15, category: 'Weapons' },
  { id: 7, name: 'Crossbow', level_required: 25, points_cost: 25, category: 'Weapons' },
  { id: 8, name: 'Flamethrower', level_required: 48, points_cost: 60, category: 'Weapons' },
  { id: 9, name: 'Assault Rifle', level_required: 25, points_cost: 30, category: 'Weapons' },
  { id: 10, name: 'Shotgun', level_required: 21, points_cost: 20, category: 'Weapons' },
  // Tools
  { id: 20, name: 'Magnifying Glass', level_required: 3, points_cost: 4, category: 'Tools' },
  { id: 21, name: 'Rope', level_required: 5, points_cost: 3, category: 'Tools' },
  { id: 22, name: 'Metal Pick', level_required: 20, points_cost: 12, category: 'Tools' },
  { id: 23, name: 'Metal Hatchet', level_required: 20, points_cost: 12, category: 'Tools' },
  { id: 24, name: 'Climbing Pick', level_required: 16, points_cost: 8, category: 'Tools' },
  { id: 25, name: 'Parachute', level_required: 15, points_cost: 8, category: 'Tools' },
  // Structures
  { id: 40, name: 'Thatch Roof', level_required: 3, points_cost: 8, category: 'Structures' },
  { id: 41, name: 'Thatch Wall', level_required: 2, points_cost: 4, category: 'Structures' },
  { id: 42, name: 'Thatch Floor', level_required: 2, points_cost: 3, category: 'Structures' },
  { id: 43, name: 'Thatch Door', level_required: 5, points_cost: 6, category: 'Structures' },
  { id: 44, name: 'Wood Roof', level_required: 15, points_cost: 12, category: 'Structures' },
  { id: 45, name: 'Wood Wall', level_required: 10, points_cost: 8, category: 'Structures' },
  { id: 46, name: 'Wood Floor', level_required: 10, points_cost: 6, category: 'Structures' },
  { id: 47, name: 'Stone Wall', level_required: 20, points_cost: 15, category: 'Structures' },
  { id: 48, name: 'Stone Foundation', level_required: 20, points_cost: 18, category: 'Structures' },
  { id: 49, name: 'Metal Wall', level_required: 40, points_cost: 25, category: 'Structures' },
  { id: 50, name: 'Metal Foundation', level_required: 40, points_cost: 30, category: 'Structures' },
  // Armor
  { id: 60, name: 'Cloth Hat', level_required: 1, points_cost: 2, category: 'Armor' },
  { id: 61, name: 'Cloth Shirt', level_required: 1, points_cost: 3, category: 'Armor' },
  { id: 62, name: 'Cloth Pants', level_required: 1, points_cost: 3, category: 'Armor' },
  { id: 63, name: 'Hide Helmet', level_required: 15, points_cost: 6, category: 'Armor' },
  { id: 64, name: 'Hide Chest', level_required: 15, points_cost: 8, category: 'Armor' },
  { id: 65, name: 'Chitin Helmet', level_required: 25, points_cost: 10, category: 'Armor' },
  { id: 66, name: 'Metal Helmet', level_required: 35, points_cost: 15, category: 'Armor' },
  { id: 67, name: 'Metal Chestpiece', level_required: 35, points_cost: 20, category: 'Armor' },
  // Crafting
  { id: 80, name: 'Campfire', level_required: 6, points_cost: 5, category: 'Crafting' },
  { id: 81, name: 'Cooking Pot', level_required: 20, points_cost: 10, category: 'Crafting' },
  { id: 82, name: 'Refining Forge', level_required: 25, points_cost: 15, category: 'Crafting' },
  { id: 83, name: 'Industrial Forge', level_required: 60, points_cost: 40, category: 'Crafting' },
  { id: 84, name: 'Mortar and Pestle', level_required: 10, points_cost: 6, category: 'Crafting' },
  { id: 85, name: 'Spinning Wheel', level_required: 15, points_cost: 8, category: 'Crafting' },
  // Saddles
  { id: 100, name: 'Raptor Saddle', level_required: 15, points_cost: 10, category: 'Saddles' },
  { id: 101, name: 'Trike Saddle', level_required: 18, points_cost: 12, category: 'Saddles' },
  { id: 102, name: 'T-Rex Saddle', level_required: 40, points_cost: 25, category: 'Saddles' },
  { id: 103, name: 'Pteranodon Saddle', level_required: 20, points_cost: 15, category: 'Saddles' },
  { id: 104, name: 'Argentavis Saddle', level_required: 50, points_cost: 30, category: 'Saddles' },
  { id: 105, name: 'Griffin Saddle', level_required: 60, points_cost: 50, category: 'Saddles' },
  // Storage
  { id: 120, name: 'Wooden Chest', level_required: 5, points_cost: 4, category: 'Storage' },
  { id: 121, name: 'Metal Storage Box', level_required: 30, points_cost: 15, category: 'Storage' },
  { id: 122, name: 'Vault', level_required: 50, points_cost: 40, category: 'Storage' },
  // Farming
  { id: 140, name: 'Crop Plot - Small', level_required: 10, points_cost: 8, category: 'Farming' },
  { id: 141, name: 'Crop Plot - Medium', level_required: 25, points_cost: 12, category: 'Farming' },
  { id: 142, name: 'Crop Plot - Large', level_required: 40, points_cost: 20, category: 'Farming' },
  { id: 143, name: 'Water Tap', level_required: 15, points_cost: 8, category: 'Farming' },
  // Electronics
  { id: 160, name: 'Wooden Transmitter', level_required: 35, points_cost: 20, category: 'Electronics' },
  { id: 161, name: 'Metal Transmitter', level_required: 50, points_cost: 30, category: 'Electronics' },
  { id: 162, name: 'Generator', level_required: 45, points_cost: 25, category: 'Electronics' },
  { id: 163, name: 'Electric Cable', level_required: 40, points_cost: 5, category: 'Electronics' },
  // Utilities
  { id: 180, name: 'Bed', level_required: 5, points_cost: 4, category: 'Utilities' },
  { id: 181, name: 'Sleeping Bag', level_required: 3, points_cost: 3, category: 'Utilities' },
  { id: 182, name: 'Torch', level_required: 3, points_cost: 2, category: 'Utilities' },
  { id: 183, name: 'Lantern', level_required: 15, points_cost: 8, category: 'Utilities' },
  { id: 184, name: 'Air Conditioner', level_required: 60, points_cost: 30, category: 'Utilities' },
]
