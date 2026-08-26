import { createFileRoute } from "@tanstack/react-router";
import img from "../assets/icons/work-in-progress.gif";
import { Button } from "@/components/ui/button";
import { Link } from "@tanstack/react-router";

export const Route = createFileRoute("/")({
    component: RouteComponent,
});

function RouteComponent() {
    return (
        <div className="flex h-screen w-screen flex-col items-center-safe justify-between bg-[url(/bg.png)] bg-cover bg-center p-8 py-20">
            <p className="font-pt-sans-caption text-center text-2xl font-bold text-white">
                Effortlessly organize stuff and connect your devices
            </p>
            <img src={img} className="aspect-square w-40 rounded-2xl" />
            <Link to="/setup">
                <Button variant="outline" className="rounded-full bg-transparent">
                    Get started
                </Button>
            </Link>
        </div>
    );
}
