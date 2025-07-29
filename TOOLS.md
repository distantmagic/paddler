    tools => vec![
        serde_json::json!({
           "type": "function",
           "function": {
               "name": "manage_zoo_animals",
               "description": "Create, read, update, or delete zoo animal records in the system.",
               "parameters": {
                   "type": "object",
                   "properties": {
                       "action": {
                           "type": "string",
                           "enum": ["create", "read", "update", "delete"],
                           "description": "The action to perform on the zoo animal record."
                       },
                       "animal": {
                           "type": "object",
                           "description": "The zoo animal record to create, read, update, or delete.",
                           "properties": {
                               "name": { "type": "string", "description": "The name of the animal." },
                               "species": { "type": "string", "description": "The species of the animal." },
                               "age": { "type": "integer", "description": "The age of the animal in years." }
                           },
                           "required": ["name", "species", "age"],
                           "additionalProperties": { "type": "string" }
                       },
                   },
                   "required": ["animal"]
               }
           }
        }),
        serde_json::json!({
           "type": "function",
           "function": {
               "name": "list_available_features",
               "description": "List all available features of the system (you).",
               "parameters": {}
           }
        })
    ],
