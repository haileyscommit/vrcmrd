import { Advisory } from "@app/bindings/Advisory";

export default function defaultAdvisory(): Advisory {
    return {
        id: "", // Generated when initialized.
        name: "", // If this is left empty when it's submitted, it'll be auto-generated based on the condition or message template.
        level: 3 as any,
        message_template: "", // Won't submit if this is empty.
        created_at: Date.now().toString(),
        updated_at: Date.now().toString(),
        created_by: null,
        updated_by: null,
        active: false,
        condition: {"type": "AllOf", "data": []},
        send_notification: false,
        send_tts: false,
        tags: [],
    };
}