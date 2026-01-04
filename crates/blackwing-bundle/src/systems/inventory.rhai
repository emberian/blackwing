// Standard Inventory System
// Handles items: pickup, drop, give, use, equip.
//
// Commands:
//   take { item: "item:key", from: "container:chest" }
//   drop { item: "item:key" }
//   give { item: "item:key", to: "npc:merchant" }
//   use_item { item: "item:potion" }
//   equip { item: "item:sword", slot: "weapon" }
//   unequip { slot: "weapon" }
//   examine { target: "item:key" }

const SYSTEM_ID = "inventory";
const HANDLES = ["take", "drop", "give", "use_item", "equip", "unequip", "examine"];
const PRIORITY = 0;

fn handle_command(cmd) {
    let kind = cmd.kind;

    if kind == "take" {
        let item_id = cmd.args["item"];
        let from_id = cmd.args["from"];

        if item_id == () {
            return;
        }

        // Check if item exists
        if !entity_exists(item_id) {
            chronicle("Inventory", "Item not found: " + item_id);
            return;
        }

        // Move item to player's inventory
        set_component(item_id, "container", "actor:player");
        add_tag(item_id, "in_inventory");

        // Get item name for chronicle
        let item_name = get_component(item_id, "name");
        if item_name == () {
            item_name = item_id;
        }

        chronicle("Inventory", "Picked up " + item_name);
    }

    if kind == "drop" {
        let item_id = cmd.args["item"];

        if item_id == () {
            return;
        }

        if !entity_exists(item_id) {
            return;
        }

        // Remove from inventory
        set_component(item_id, "container", ());
        remove_tag(item_id, "in_inventory");
        remove_tag(item_id, "equipped");

        let item_name = get_component(item_id, "name");
        if item_name == () {
            item_name = item_id;
        }

        chronicle("Inventory", "Dropped " + item_name);
    }

    if kind == "give" {
        let item_id = cmd.args["item"];
        let to_id = cmd.args["to"];

        if item_id == () || to_id == () {
            return;
        }

        if !entity_exists(item_id) || !entity_exists(to_id) {
            return;
        }

        // Transfer item
        set_component(item_id, "container", to_id);
        remove_tag(item_id, "in_inventory");
        remove_tag(item_id, "equipped");

        let item_name = get_component(item_id, "name");
        if item_name == () {
            item_name = item_id;
        }

        let target_name = get_component(to_id, "name");
        if target_name == () {
            target_name = to_id;
        }

        chronicle("Inventory", "Gave " + item_name + " to " + target_name);
    }

    if kind == "use_item" {
        let item_id = cmd.args["item"];

        if item_id == () {
            return;
        }

        if !entity_exists(item_id) {
            return;
        }

        // Check if item is usable
        let usable = get_component(item_id, "usable");
        if usable != true {
            chronicle("Inventory", "Cannot use this item");
            return;
        }

        // Check if consumable
        let consumable = get_component(item_id, "consumable");

        let item_name = get_component(item_id, "name");
        if item_name == () {
            item_name = item_id;
        }

        // Apply item effects (game-specific logic would go here via hooks)
        chronicle("Inventory", "Used " + item_name);

        // Remove if consumable
        if consumable == true {
            despawn_entity(item_id);
        }
    }

    if kind == "equip" {
        let item_id = cmd.args["item"];
        let slot = cmd.args["slot"];

        if item_id == () {
            return;
        }

        if !entity_exists(item_id) {
            return;
        }

        // Check if item is equippable
        let equippable = get_component(item_id, "equippable");
        if equippable != true {
            chronicle("Inventory", "Cannot equip this item");
            return;
        }

        // Equip the item
        add_tag(item_id, "equipped");
        if slot != () {
            set_component(item_id, "equipped_slot", slot);
        }

        let item_name = get_component(item_id, "name");
        if item_name == () {
            item_name = item_id;
        }

        chronicle("Inventory", "Equipped " + item_name);
    }

    if kind == "unequip" {
        let slot = cmd.args["slot"];
        let item_id = cmd.args["item"];

        // Find equipped item by slot or id
        // For now, just use item_id if provided
        if item_id != () && entity_exists(item_id) {
            remove_tag(item_id, "equipped");
            set_component(item_id, "equipped_slot", ());

            let item_name = get_component(item_id, "name");
            if item_name == () {
                item_name = item_id;
            }

            chronicle("Inventory", "Unequipped " + item_name);
        }
    }

    if kind == "examine" {
        let target_id = cmd.args["target"];

        if target_id == () {
            return;
        }

        if !entity_exists(target_id) {
            chronicle("Examine", "Nothing to examine");
            return;
        }

        let name = get_component(target_id, "name");
        let description = get_component(target_id, "description");

        if name == () {
            name = target_id;
        }
        if description == () {
            description = "You see nothing special.";
        }

        // Show examination UI
        show_modal(vstack([
            text(name),
            passage(description),
            button("Close", `{"kind":"close_modal"}`)
        ]));

        chronicle("Examine", "Examined " + name);
    }
}
