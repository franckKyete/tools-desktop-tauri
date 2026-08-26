import { createFileRoute } from "@tanstack/react-router";
import { QRCodeCanvas } from "qrcode.react"; // or QRCodeSVG
// import img from "../assets/imgs/qrcode.svg";

import { invoke } from "@tauri-apps/api/core";

export const Route = createFileRoute("/setup")({
    component: RouteComponent,
    loader: async () => {
        return await invoke<string>("get_advertisement", {});
    },
});

function RouteComponent() {
    const advert = Route.useLoaderData();

    return (
        <div className="flex h-screen w-screen flex-col items-center-safe justify-center gap-10 bg-[url(/bg.png)] bg-cover bg-center p-8 py-20">
            <p className="text-xl font-bold">Scan the QR code</p>
            <QRCodeCanvas
                className="rounded-2xl"
                value={advert}
                size={200}
                bgColor={"#ffffff"}
                fgColor={"#000000"}
                level={"H"} // Error Correction Level: L, M, Q, H
                includeMargin={true}
            />
            {/* <img className="aspect-square w-55 rounded-2xl" src={img} /> */}
            <p className="font-bold">Kyete-hp-desktop</p>
        </div>
    );
}
