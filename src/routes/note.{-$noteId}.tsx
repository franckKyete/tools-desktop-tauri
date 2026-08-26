import { createFileRoute, useRouter } from "@tanstack/react-router";

// import "@blocknote/core/fonts/inter.css";
import { BlockNoteView } from "@blocknote/shadcn";
import "@blocknote/shadcn/style.css";
import {
    DefaultReactSuggestionItem,
    getDefaultReactSlashMenuItems,
    SuggestionMenuController,
    useCreateBlockNote,
} from "@blocknote/react";
import "../styles/blocknote.css";
import { BlockNoteEditor, filterSuggestionItems } from "@blocknote/core";
import { Button } from "@/components/ui/button";
import { X } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { Note } from "@/utils";
import { useMemo } from "react";
// import { HiOutlineGlobeAlt } from "react-icons/hi";

export const Route = createFileRoute("/note/{-$noteId}")({
    component: RouteComponent,
    loader: async ({ params }) => {
        if (params.noteId !== undefined) {
            return await invoke<Note>("get_note", { id: params.noteId });
        }
        return await invoke<Note>("new_note");
    },
});

function RouteComponent() {
    const note = Route.useLoaderData();

    const initialContent = useMemo(() => {
        try {
            return JSON.parse(note.document);
        } catch {
            return undefined;
        }
    }, [note]);

    const router = useRouter();
    const editor = useCreateBlockNote({
        initialContent,
    });

    return (
        <div className="flex h-screen w-screen flex-col overflow-clip bg-[url(/bg.png)] bg-cover bg-center">
            <div className="mt-3 mb-10 flex justify-end">
                <Button
                    onClick={async () => {
                        invoke("update_note", {
                            id: note.id,
                            doc: JSON.stringify(editor.document),
                        });
                        router.history.back();
                    }}
                    variant="link"
                >
                    <X />
                </Button>
            </div>
            <BlockNoteView
                sideMenu={false}
                className="w-full"
                theme="dark"
                editor={editor}
                slashMenu={false}
            >
                <SuggestionMenuController
                    triggerCharacter={"/"}
                    // Replaces the default Slash Menu items with our custom ones.
                    getItems={async (query) =>
                        filterSuggestionItems(getCustomSlashMenuItems(editor), query)
                    }
                />
            </BlockNoteView>
        </div>
    );
}

// Custom Slash Menu item to insert a block after the current one.
// export const insertHelloWorldItem = (editor: BlockNoteEditor) => ({
//     title: "Insert Hello World",
//     onItemClick: () =>
//         // If the block containing the text caret is empty, `insertOrUpdateBlock`
//         // changes its type to the provided block. Otherwise, it inserts the new
//         // block below and moves the text caret to it. We use this function with
//         // a block containing 'Hello World' in bold.
//         insertOrUpdateBlock(editor, {
//             type: "paragraph",
//             content: [{ type: "text", text: "Hello World", styles: { bold: true } }],
//         }),
//     aliases: ["helloworld", "hw"],
//     group: "Other",
//     icon: <HiOutlineGlobeAlt size={18} />,
//     subtext: "Used to insert a block with 'Hello World' below.",
// });

// List containing all default Slash Menu Items, as well as our custom one.
const getCustomSlashMenuItems = (
    editor: BlockNoteEditor,
): DefaultReactSuggestionItem[] => [
        ...getDefaultReactSlashMenuItems(editor)
            .filter((item) => {
                return [
                    "Heading 3",
                    "Check List",
                    "Bullet List",
                    "Code Block",
                    "Divider",
                    "Image",
                    "File",
                    "Emoji",
                ].includes(item.title);
            })
            .map((item) => ({
                ...item,
                group: " ",
            })),
        // insertHelloWorldItem(editor),
    ];
