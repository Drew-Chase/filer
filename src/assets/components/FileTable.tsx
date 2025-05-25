import {Button, Dropdown, DropdownItem, DropdownMenu, DropdownSection, DropdownTrigger, Pagination, Spinner, Table, TableBody, TableCell, TableColumn, TableHeader, TableRow} from "@heroui/react";
import {FileSystem, FilesystemEntry} from "../ts/filesystem.ts";
import {Icon} from "@iconify-icon/react";
import {useFileSystemEntry} from "../providers/FileSystemEntryProvider.tsx";
import FileEntryIcon from "./FileEntryIcon.tsx";
import {motion} from "framer-motion";
import {useEffect, useState} from "react";
import {useFavorites} from "../providers/FavoritesProvider.tsx";
import {useLocation} from "react-router-dom";

export default function FileTable()
{
    const {
        navigate,
        loading,
        data,
        sortDescriptor,
        onSortChange,
        openRenameModal,
        openCopyModal,
        openMoveModal,
        openDeleteModal,
        downloadEntry,
        selectedEntries,
        setSelectedEntries,
        currentDirectoryFilter
    } = useFileSystemEntry();
    const {pathname} = useLocation();


    const MAX_ITEMS_PER_PAGE = 25;
    const [pageItems, setPageItems] = useState<FilesystemEntry[]>([]);
    const [currentPage, setCurrentPage] = useState(1);
    const {addFavorite, removeFavorite, isFavorited} = useFavorites();

    useEffect(() =>
    {
        setPageItems(data.entries.filter(i => i.filename.toLowerCase().includes(currentDirectoryFilter.toLowerCase())).slice((currentPage - 1) * MAX_ITEMS_PER_PAGE, currentPage * MAX_ITEMS_PER_PAGE));
        setSelectedEntries(new Set());
    }, [data, currentPage, sortDescriptor, currentDirectoryFilter]);

    useEffect(() =>
    {
        setCurrentPage(1);
    }, [pathname]);

    return (
        <>
            <Table
                removeWrapper
                isHeaderSticky
                aria-label={"File table"}
                className={"w-full"}
                classNames={{
                    base: "w-full max-h-[calc(100dvh_-_180px)] overflow-y-auto",
                    th: "!bg-white/10 backdrop-contrast-105 backdrop-brightness-75 backdrop-blur-lg",
                    td: "group-aria-[selected=false]/tr:group-data-[hover=true]/tr:before:bg-white/10 before:bg-white/10 before:transition-all before:duration-200"
                }}
                sortDescriptor={sortDescriptor}
                onSortChange={onSortChange}
                selectionMode={data.entries.length > 0 ? "multiple" : "none"}
                selectedKeys={new Set(
                    Array.from(selectedEntries).map(selectedEntry =>
                    {
                        // Find the index of the entry with this selectedEntry
                        const index = data.entries.findIndex(entry => entry.path === selectedEntry.path);
                        return index >= 0 ? index.toString() : "";
                    }).filter(index => index !== "")
                )}
                onSelectionChange={(keys) =>
                {
                    if (keys === "all")
                    {
                        setSelectedEntries(new Set(pageItems));
                        return;
                    }
                    // Convert the keys to a Set of strings
                    const selectedKeys = new Set(Array.from(keys).map(key =>
                    {
                        // Find the entry with this key (index)
                        const index = parseInt(key.toString());
                        const path = data.entries[index]?.path;
                        if (isNaN(index) || path === undefined) return null;

                        return data.entries[index] ?? null;
                    }).filter(path => path != null));

                    setSelectedEntries(selectedKeys);
                }}
            >
                <TableHeader>
                    <TableColumn key={"filename"} className="w-full" allowsSorting aria-label="Name column">Name</TableColumn>
                    <TableColumn key={"is_dir"} className="min-w-32" allowsSorting aria-label="Type column">Type</TableColumn>
                    <TableColumn key={"size"} className="min-w-32" allowsSorting aria-label="Size column">Size</TableColumn>
                    <TableColumn key={"creation_date"} className="min-w-32" allowsSorting aria-label="Creation date column">Creation</TableColumn>
                    <TableColumn key={"last_modified"} className="min-w-32" allowsSorting aria-label="Modification date column">Modification</TableColumn>
                    <TableColumn className="text-right" aria-label="Actions column">Actions</TableColumn>
                </TableHeader>
                <TableBody isLoading={loading} loadingContent={<Spinner color={"primary"}/>}>
                    {pageItems.map((entry, index) => (
                        <TableRow key={index} aria-label={`File entry: ${entry.filename}`}>
                            <TableCell className="font-medium" aria-label="File name">
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
                                        aria-label={`Open directory ${entry.filename}`}
                                    >
                                        <Icon
                                            icon={entry.is_dir ? "mage:folder-fill" : "mage:file-fill"}
                                            className={"text-2xl data-[directory=true]:text-blue-500"}
                                            data-directory={entry.is_dir ? "true" : "false"}
                                            aria-hidden="true"
                                        />
                                        {entry.filename}
                                    </Button>
                                    : <div className={"text-tiny flex flex-row items-center px-3 gap-2"} aria-label={`File ${entry.filename}`}>
                                        <FileEntryIcon entry={entry}/>
                                        {entry.filename}
                                    </div>
                                }
                            </TableCell>
                            <TableCell aria-label="Entry type">{entry.file_type ?? "Unknown Entry Type"}</TableCell>
                            <TableCell aria-label="File size">{entry.is_dir ? "-" : FileSystem.formatSize(entry.size)}</TableCell>
                            <TableCell aria-label="Creation date">{entry.is_dir ? "-" : entry.creation_date.toLocaleDateString()}</TableCell>
                            <TableCell aria-label="Modification date">{entry.is_dir ? "-" : entry.last_modified.toLocaleDateString()}</TableCell>
                            <TableCell className="text-right" aria-label="Actions">
                                <Dropdown
                                    classNames={{
                                        content: "bg-[#2f115c]/50 outline-primary outline-1 backdrop-brightness-[.5] backdrop-saturate-[1.5] backdrop-blur-lg"
                                    }}
                                    aria-label={`Actions for ${entry.filename}`}
                                >
                                    <DropdownTrigger>
                                        <Button variant={"light"} size={"sm"} aria-label={`Open actions menu for ${entry.filename}`}>
                                            <Icon icon={"mage:dots-horizontal"} aria-hidden="true"/>
                                        </Button>
                                    </DropdownTrigger>
                                    <DropdownMenu
                                        aria-label={`Available actions for ${entry.filename}`}
                                        itemClasses={{
                                            base: "data-[hover=true]:bg-white/10 data-[selectable=true]:focus:bg-white/50"
                                        }}
                                    >
                                        <DropdownSection title={`${entry.filename} options`} showDivider>
                                            {entry.is_dir ? (
                                                <DropdownItem
                                                    key={`favorite-${entry.filename}`}
                                                    onPress={() =>
                                                    {
                                                        if (isFavorited(entry.path))
                                                            removeFavorite(entry.path);
                                                        else
                                                            addFavorite({path: entry.path, name: entry.filename});

                                                    }}
                                                >
                                                    {isFavorited(entry.path) ? "UnFavorite" : "Favorite"}
                                                </DropdownItem>
                                            ) : (<></>)}
                                            <DropdownItem key={`rename-${entry.filename}`} endContent={<Icon icon={"gg:rename"} width={18} aria-hidden="true"/>} onPress={() => openRenameModal(entry)} aria-label={`Rename ${entry.filename}`}>Rename</DropdownItem>
                                            <DropdownItem key={`copy-${entry.filename}`} endContent={<Icon icon={"mage:copy-fill"} aria-hidden="true"/>} onPress={() => openCopyModal([entry])} aria-label={`Copy ${entry.filename}`}>Copy</DropdownItem>
                                            <DropdownItem key={`move-${entry.filename}`} endContent={<Icon icon={"mage:l-arrow-right-up"} width={18} aria-hidden="true"/>} onPress={() => openMoveModal([entry])} aria-label={`Move ${entry.filename}`}>Move</DropdownItem>
                                            <DropdownItem key={`share-${entry.filename}`} endContent={<Icon icon={"mage:share-fill"} width={16} aria-hidden="true"/>} aria-label={`Share ${entry.filename}`}>Share</DropdownItem>
                                            <DropdownItem key={`download-${entry.filename}`} endContent={<Icon icon={"mage:file-download-fill"} aria-hidden="true"/>} onPress={() => downloadEntry(entry)} aria-label={`Download ${entry.filename}`}>Download</DropdownItem>
                                        </DropdownSection>
                                        <DropdownSection title={"danger zone"} className={"text-danger"}>
                                            <DropdownItem
                                                key={`delete-${entry.filename}`}
                                                endContent={<Icon icon={"mage:trash-fill"} aria-hidden="true"/>}
                                                color={"danger"}
                                                onPress={() => openDeleteModal([entry])}
                                                aria-label={`Delete ${entry.filename}`}
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
                        <TableRow aria-label={"No files found"}>
                            <TableCell colSpan={6} className="text-center py-8" aria-label="No files found">
                                {data?.entries.length === 0 ? "This directory is empty" : "No matching files found"}
                            </TableCell>
                        </TableRow>
                    ) as any}
                </TableBody>
            </Table>
            <motion.div
                className={"absolute bottom-12 left-1/2 bg-white/10 p-3 backdrop-blur-sm backdrop-contrast-[1.1] rounded-2xl"}
                initial={{opacity: 0, y: 20}}
                animate={{opacity: data.entries.length > MAX_ITEMS_PER_PAGE ? 1 : 0, y: data.entries.length > MAX_ITEMS_PER_PAGE ? 0 : 20, x: "-50%"}}
                transition={{duration: 1, delay: 0, type: "spring", ease: "easeInOut"}}
            >
                <Pagination
                    total={Math.ceil(data.entries.length / MAX_ITEMS_PER_PAGE)}
                    page={currentPage}
                    onChange={setCurrentPage}
                    size={"sm"}

                    showControls
                    showShadow
                    classNames={{
                        item: "bg-white text-black data-[hover]:!bg-white/80",
                        next: "bg-white text-black data-[hover]:!bg-white/80",
                        prev: "bg-white text-black data-[hover]:!bg-white/80"
                    }}
                />
            </motion.div>
        </>
    );
}
