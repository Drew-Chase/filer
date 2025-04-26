import {Button, Dropdown, DropdownItem, DropdownMenu, DropdownSection, DropdownTrigger, Spinner, Table, TableBody, TableCell, TableColumn, TableHeader, TableRow} from "@heroui/react";
import fs from "../ts/filesystem.ts";
import {Icon} from "@iconify-icon/react";
import {useFileSystemEntry} from "../providers/FileSystemEntryProvider.tsx";

export default function FileTable()
{
    const {navigate, loading, data, sortDescriptor, onSortChange} = useFileSystemEntry();

    return (
        <Table
            removeWrapper
            isHeaderSticky
            className={"w-full"}
            classNames={{
                base: "w-full max-h-[calc(100dvh_-_180px)] overflow-y-auto",
                th: "!bg-black/10 dark:!bg-white/10 backdrop-blur-sm"
            }}
            sortDescriptor={sortDescriptor}
            onSortChange={onSortChange}
            selectionMode={data.entries.length > 0 ? "multiple" : "none"}
        >
            <TableHeader>
                <TableColumn key={"filename"} className="w-full" allowsSorting>Name</TableColumn>
                <TableColumn key={"is_dir"} className="min-w-32" allowsSorting>Type</TableColumn>
                <TableColumn key={"size"} className="min-w-32" allowsSorting>Size</TableColumn>
                <TableColumn key={"creation_date"} className="min-w-32" allowsSorting>Creation</TableColumn>
                <TableColumn key={"modification_date"} className="min-w-32" allowsSorting>Modification</TableColumn>
                <TableColumn className="text-right">Actions</TableColumn>
            </TableHeader>
            <TableBody isLoading={loading} loadingContent={<Spinner color={"primary"}/>}>
                {data?.parent && (
                    <TableRow>
                        <TableCell className="font-medium">
                            <Button
                                onPress={() => navigate(data.parent || "/")}
                                variant={"light"}
                                size={"sm"}
                                className={`text-start justify-start w-full`}
                            >
                                <Icon icon={"mage:folder-fill"} className={"text-2xl text-blue-500"}/> ../ (Parent Directory)
                            </Button>
                        </TableCell>
                        <TableCell>Directory</TableCell>
                        <TableCell>-</TableCell>
                        <TableCell>-</TableCell>
                        <TableCell>-</TableCell>
                        <TableCell className="text-right">-</TableCell>
                    </TableRow>
                )}

                {data.entries.map((entry, index) => (
                    <TableRow key={index}>
                        <TableCell className="font-medium">
                            {entry.is_dir ?
                                <Button
                                    onPress={() =>
                                    {
                                        if (entry.is_dir)
                                            navigate(entry.path);
                                    }}
                                    variant={"light"}
                                    size={"sm"}
                                    className={`text-start justify-start w-full`}
                                >
                                    <Icon
                                        icon={entry.is_dir ? "mage:folder-fill" : "mage:file-fill"}
                                        className={"text-2xl data-[directory=true]:text-blue-500"}
                                        data-directory={entry.is_dir ? "true" : "false"}
                                    />
                                    {entry.filename}
                                </Button>
                                : <div className={"text-tiny flex flex-row items-center px-3 gap-2"}>
                                    <Icon
                                        icon={entry.is_dir ? "mage:folder-fill" : "mage:file-fill"}
                                        className={"text-2xl data-[directory=true]:text-blue-500"}
                                        data-directory={entry.is_dir ? "true" : "false"}
                                    />
                                    {entry.filename}
                                </div>
                            }
                        </TableCell>
                        <TableCell>{entry.is_dir ? "Directory" : "File"}</TableCell>
                        <TableCell>{entry.is_dir ? "-" : fs.formatSize(entry.size)}</TableCell>
                        <TableCell>{entry.is_dir ? "-" : entry.creation_date.toLocaleDateString()}</TableCell>
                        <TableCell>{entry.is_dir ? "-" : entry.last_modified.toLocaleDateString()}</TableCell>
                        <TableCell className="text-right">
                            <Dropdown>
                                <DropdownTrigger>
                                    <Button variant={"light"} size={"sm"}>
                                        <Icon icon={"mage:dots-horizontal"}/>
                                    </Button>
                                </DropdownTrigger>
                                <DropdownMenu>
                                    <DropdownSection title={`${entry.filename} options`} showDivider>
                                        <DropdownItem key={`download-${entry.filename}`} endContent={<Icon icon={"gg:rename"} width={18}/>}>Rename</DropdownItem>
                                        <DropdownItem key={`download-${entry.filename}`} endContent={<Icon icon={"mage:copy-fill"}/>}>Copy</DropdownItem>
                                        <DropdownItem key={`download-${entry.filename}`} endContent={<Icon icon={"mage:l-arrow-right-up"} width={18}/>}>Move</DropdownItem>
                                        <DropdownItem key={`download-${entry.filename}`} endContent={<Icon icon={"mage:share-fill"} width={16}/>}>Share</DropdownItem>
                                        <DropdownItem key={`download-${entry.filename}`} endContent={<Icon icon={"mage:file-download-fill"}/>}>Download</DropdownItem>
                                    </DropdownSection>
                                    <DropdownSection title={"danger zone"} className={"text-danger"}>
                                        <DropdownItem
                                            key={`delete-${entry.filename}`}
                                            endContent={<Icon icon={"mage:trash-fill"}/>}
                                            color={"danger"}
                                        >
                                            Delete
                                        </DropdownItem>
                                    </DropdownSection>
                                </DropdownMenu>
                            </Dropdown>
                        </TableCell>
                    </TableRow>
                ))}

                {data.entries.length === 0 && !loading && (
                    <TableRow>
                        <TableCell colSpan={6} className="text-center py-8">
                            {data?.entries.length === 0 ? "This directory is empty" : "No matching files found"}
                        </TableCell>
                    </TableRow>
                ) as any}
            </TableBody>
        </Table>
    );
}