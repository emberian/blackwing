// Standard Dialogue System
// Handles conversation trees with branching choices.
//
// Commands:
//   start_dialogue { dialogue: "dialogue_id", speaker: "npc:name" }
//   dialogue_choice { index: 0, target: "node_id" }
//   end_dialogue {}

const SYSTEM_ID = "dialogue";
const HANDLES = ["start_dialogue", "dialogue_choice", "end_dialogue"];
const PRIORITY = 0;

fn handle_command(cmd) {
    let kind = cmd.kind;

    if kind == "start_dialogue" {
        let dialogue_id = cmd.args["dialogue"];
        let speaker = cmd.args["speaker"];

        // Push conversation scope
        push_scope("conversation", dialogue_id);
        set_scope_local("node", "start");
        if speaker != () {
            set_scope_local("speaker", speaker);
        }

        // Build initial dialogue UI
        // Note: In a full implementation, this would look up the dialogue content
        // For now, we set a simple placeholder that the frontend can interpret
        set_ui(vstack([
            passage("(Dialogue started: " + dialogue_id + ")"),
            choices([
                #{ text: "Continue", action: `{"kind":"dialogue_choice","args":{"target":"end"}}` }
            ])
        ]));

        chronicle("Dialogue", "Started conversation: " + dialogue_id);
    }

    if kind == "dialogue_choice" {
        let target = cmd.args["target"];

        if target == "end" || target == () {
            // End the dialogue
            pop_scope();
            clear_ui();
            chronicle("Dialogue", "Ended conversation");
        } else {
            // Move to the next node
            set_scope_local("node", target);

            // Update UI for new node
            set_ui(vstack([
                passage("(Dialogue node: " + target + ")"),
                choices([
                    #{ text: "Continue", action: `{"kind":"dialogue_choice","args":{"target":"end"}}` }
                ])
            ]));
        }
    }

    if kind == "end_dialogue" {
        pop_scope();
        clear_ui();
        chronicle("Dialogue", "Ended conversation");
    }
}
