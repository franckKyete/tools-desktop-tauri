import { createFileRoute, Link, useRouter } from "@tanstack/react-router";
import { Copy, Pin, Plus, Search, Tag, Trash2, X } from "lucide-react";
import { Button } from "@/components/ui/button";
import { invoke } from "@tauri-apps/api/core";
import { Note } from "@/utils";
import { useCreateBlockNote } from "@blocknote/react";
import { useState } from "react";

export const Route = createFileRoute("/notes")({
    component: RouteComponent,
    loader: async () => {
        return await invoke<Record<string, Note>>("get_notes", {});
    },
});

function RouteComponent() {
    const router = useRouter();

    const notes = Route.useLoaderData();
    const editor = useCreateBlockNote();

    const [contextMenuShown, setContextMenuShown] = useState(false);

    return (
        <div className="relative flex h-screen w-screen flex-col gap-10 overflow-clip bg-[url(/bg.png)] bg-cover bg-center p-8 pt-20">
            {!contextMenuShown && (
                <search className="flex gap-2 rounded-full border border-[#1b1b1b] bg-black/10 px-5 py-2 backdrop-blur-xs">
                    <Search color="#666666" />
                    <input
                        className="h-full w-full py-1"
                        type="search"
                        placeholder="Search ..."
                    />
                </search>
            )}
            {contextMenuShown && (
                <div className="flex h-12 w-full items-center gap-5 bg-black/40 p-3">
                    <button
                        className="mr-auto"
                        onClick={() => setContextMenuShown(false)}
                    >
                        <X size={18} />
                    </button>
                    <button>
                        <Pin size={18} />
                    </button>
                    <button>
                        <Trash2 size={18} />
                    </button>
                    <button>
                        <Tag size={18} />
                    </button>
                    <button>
                        <Copy size={18} />
                    </button>
                </div>
            )}
            <div className="grid-rows-masonry grid grid-cols-2 gap-5 overflow-scroll">
                {Object.entries(notes).map(async ([key, note]) => (
                    <Link to="/note/{-$noteId}" params={{ noteId: key }}>
                        <div
                            key={key}
                            className="line-clamp-4 max-h-40 min-h-15 rounded-xl border border-[#1B1B1B] bg-black/10 p-3 text-sm backdrop-blur-xs"
                            onContextMenu={(e) => {
                                e.preventDefault();
                                setContextMenuShown(true);
                            }}
                        >
                            <div
                                className="line-clamp-6"
                                dangerouslySetInnerHTML={{
                                    __html: editor.blocksToFullHTML(JSON.parse(note.document)),
                                }}
                            ></div>
                        </div>
                    </Link>
                ))}
            </div>
            <Button
                onClick={() => {
                    router.navigate({
                        to: "/note/{-$noteId}",
                    });
                }}
                className="absolute right-10 bottom-10 cursor-pointer"
                size="icon"
            >
                <Plus size={48} />
            </Button>
        </div>
    );
}
