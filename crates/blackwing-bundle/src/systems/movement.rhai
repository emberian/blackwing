// Standard Movement System
// Handles navigation between locations.
//
// Commands:
//   go { direction: "north" }
//   travel { destination: "location:town" }
//   look { target: "location:current" }
//   enter { location: "location:building" }
//   exit {}

const SYSTEM_ID = "movement";
const HANDLES = ["go", "travel", "look", "enter", "exit"];
const PRIORITY = 0;

fn handle_command(cmd) {
    let kind = cmd.kind;

    if kind == "go" {
        let direction = cmd.args["direction"];

        if direction == () {
            chronicle("Movement", "Go where?");
            return;
        }

        // Get player's current position
        let current_pos = get_component("actor:player", "position");
        if current_pos == () {
            chronicle("Movement", "You don't know where you are");
            return;
        }

        // Get exits from current location
        let exits = get_component(current_pos, "exits");
        if exits == () {
            chronicle("Movement", "There are no exits here");
            return;
        }

        // Check if direction is valid
        let destination = exits[direction];
        if destination == () {
            chronicle("Movement", "You cannot go " + direction);
            return;
        }

        // Move player
        set_component("actor:player", "position", destination);

        // Push location scope
        push_scope("location", destination);

        // Get destination name
        let dest_name = get_component(destination, "name");
        if dest_name == () {
            dest_name = destination;
        }

        chronicle("Movement", "Went " + direction + " to " + dest_name);

        // Trigger a look at the new location
        let description = get_component(destination, "description");
        if description != () {
            set_ui(vstack([
                text(dest_name),
                passage(description)
            ]));
        }
    }

    if kind == "travel" {
        let destination = cmd.args["destination"];

        if destination == () {
            chronicle("Movement", "Travel where?");
            return;
        }

        if !entity_exists(destination) {
            chronicle("Movement", "Unknown destination: " + destination);
            return;
        }

        // Check if destination is reachable (game can add more complex checks)
        let reachable = get_component(destination, "reachable");
        if reachable == false {
            chronicle("Movement", "Cannot travel there right now");
            return;
        }

        // Move player
        set_component("actor:player", "position", destination);

        // Push location scope
        push_scope("location", destination);

        // Get destination name
        let dest_name = get_component(destination, "name");
        if dest_name == () {
            dest_name = destination;
        }

        chronicle("Movement", "Traveled to " + dest_name);

        // Show the new location
        let description = get_component(destination, "description");
        if description != () {
            set_ui(vstack([
                text(dest_name),
                passage(description)
            ]));
        }
    }

    if kind == "look" {
        let target = cmd.args["target"];

        // Default to current location
        if target == () {
            target = get_component("actor:player", "position");
        }

        if target == () {
            chronicle("Look", "You look around but see nothing");
            return;
        }

        if !entity_exists(target) {
            chronicle("Look", "Nothing to look at");
            return;
        }

        let name = get_component(target, "name");
        let description = get_component(target, "description");

        if name == () {
            name = target;
        }
        if description == () {
            description = "You see nothing special.";
        }

        // Show location UI
        set_ui(vstack([
            text(name),
            passage(description)
        ]));

        chronicle("Look", "Looked at " + name);
    }

    if kind == "enter" {
        let location = cmd.args["location"];

        if location == () {
            chronicle("Movement", "Enter what?");
            return;
        }

        if !entity_exists(location) {
            chronicle("Movement", "Cannot find: " + location);
            return;
        }

        // Check if enterable
        let enterable = get_component(location, "enterable");
        if enterable == false {
            chronicle("Movement", "Cannot enter there");
            return;
        }

        // Store previous location for exit
        let current_pos = get_component("actor:player", "position");
        set_component("actor:player", "previous_position", current_pos);

        // Move player
        set_component("actor:player", "position", location);

        // Push interior scope
        push_scope("interior", location);

        let loc_name = get_component(location, "name");
        if loc_name == () {
            loc_name = location;
        }

        chronicle("Movement", "Entered " + loc_name);

        // Show the interior
        let description = get_component(location, "description");
        if description != () {
            set_ui(vstack([
                text(loc_name),
                passage(description)
            ]));
        }
    }

    if kind == "exit" {
        // Get previous position
        let prev_pos = get_component("actor:player", "previous_position");

        if prev_pos == () {
            chronicle("Movement", "There's nowhere to exit to");
            return;
        }

        // Pop interior scope
        pop_scope();

        // Move back
        set_component("actor:player", "position", prev_pos);
        set_component("actor:player", "previous_position", ());

        let loc_name = get_component(prev_pos, "name");
        if loc_name == () {
            loc_name = prev_pos;
        }

        chronicle("Movement", "Exited to " + loc_name);

        // Show the location
        let description = get_component(prev_pos, "description");
        if description != () {
            set_ui(vstack([
                text(loc_name),
                passage(description)
            ]));
        }
    }
}
