import {BreadcrumbItem, Breadcrumbs} from "@heroui/react";
import {useFileSystemEntry} from "../providers/FileSystemEntryProvider.tsx";

type FileTableBreadcrumbsProperties = {
    onNavigate?: (path: string) => void;
    paths: string[];
}

export default function FileTableBreadcrumbs(props: FileTableBreadcrumbsProperties)
{
    const {navigate} = useFileSystemEntry();
    const {paths, onNavigate} = props;

    return (
        <Breadcrumbs variant={"bordered"}>
            {(!paths.length || paths.every(p => p.trim() === "")) ? (
                <BreadcrumbItem
                    key="root"
                    onPress={() =>
                    {
                        if (onNavigate) onNavigate("/");
                        else navigate("/");
                    }}
                >
                    Root
                </BreadcrumbItem>
            ) : (
                paths.filter(i => i.trim() !== "").map((path, index) => (
                    <BreadcrumbItem
                        key={path}
                        onPress={() =>
                        {
                            if (index >= paths.length)
                                return;
                            let newPath = paths.slice(0, index + 1).join("/");
                            if (onNavigate) onNavigate(newPath);
                            else navigate(newPath);
                        }}
                    >
                        {path === "" || path === "/" ? "Root" : path}
                    </BreadcrumbItem>
                ))
            )}
        </Breadcrumbs>
    );
}