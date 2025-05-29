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
            {paths.slice(0, paths.length - 1).map((path, index) => (
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
            ))}
        </Breadcrumbs>
    );
}