import {BreadcrumbItem, Breadcrumbs} from "@heroui/react";
import {useFileSystemEntry} from "../providers/FileSystemEntryProvider.tsx";

export default function FileTableBreadcrumbs({paths}: { paths: string[] })
{
    const {navigate} = useFileSystemEntry();
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
                        navigate(newPath);
                    }}
                >
                    {path === "" || path === "/" ? "Root" : path}
                </BreadcrumbItem>
            ))}
        </Breadcrumbs>
    );
}