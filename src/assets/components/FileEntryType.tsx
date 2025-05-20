import {FilesystemEntry} from "../ts/filesystem.ts";

export default function FileEntryType({entry}: { entry: FilesystemEntry })
{
    // Return "Directory" if the entry is a directory
    if (entry.is_dir) {
        return (<>Directory</>);
    }
    
    const extensions = entry.filename.toLowerCase().trim().split(".").slice(1);
    let extension = extensions.length > 0 ? extensions.join(".") : "";
    
    switch (extension)
    {
        // Microsoft Office Documents
        case "doc":
        case "docm":
        case "docx":
            return (<>Word Document</>);
        case "dot":
        case "dotx":
        case "dotm":
            return (<>Word Template</>);
        case "xls":
        case "xlsx":
        case "xlsm":
        case "xlsb":
            return (<>Excel Spreadsheet</>);
        case "xlt":
        case "xltx":
        case "xltm":
        case "xlw":
            return (<>Excel Template</>);
        case "ppt":
        case "pptx":
        case "pptm":
            return (<>PowerPoint Presentation</>);
        case "pot":
        case "potx":
        case "potm":
            return (<>PowerPoint Template</>);
        
        // Open Document Formats
        case "odt":
            return (<>OpenDocument Text</>);
        case "ods":
            return (<>OpenDocument Spreadsheet</>);
        case "odp":
            return (<>OpenDocument Presentation</>);
        
        // PDF Documents
        case "pdf":
            return (<>PDF Document</>);
        case "fdf":
        case "xfdf":
        case "pdx":
        case "xdp":
            return (<>PDF Data File</>);
        
        // Images
        case "jpg":
        case "jpeg":
            return (<>JPEG Image</>);
        case "png":
            return (<>PNG Image</>);
        case "gif":
            return (<>GIF Image</>);
        case "bmp":
            return (<>Bitmap Image</>);
        case "svg":
            return (<>SVG Vector Image</>);
        case "webp":
            return (<>WebP Image</>);
        case "tiff":
        case "tif":
            return (<>TIFF Image</>);
        case "ico":
            return (<>Icon File</>);
        case "psd":
        case "psb":
        case "pdd":
            return (<>Photoshop Document</>);
        case "ai":
        case "ait":
        case "art":
        case "aip":
            return (<>Illustrator Document</>);
        case "indd":
        case "indl":
        case "indt":
        case "indb":
            return (<>InDesign Document</>);
        
        // Video Files
        case "mp4":
        case "mpeg4":
            return (<>MP4 Video</>);
        case "webm":
            return (<>WebM Video</>);
        case "avi":
            return (<>AVI Video</>);
        case "mov":
        case "qt":
            return (<>QuickTime Video</>);
        case "mkv":
            return (<>Matroska Video</>);
        case "flv":
            return (<>Flash Video</>);
        case "wmv":
            return (<>Windows Media Video</>);
        case "mpeg":
        case "mpg":
            return (<>MPEG Video</>);
        case "m4v":
            return (<>M4V Video</>);
        case "3gp":
            return (<>3GP Video</>);
        case "ogv":
            return (<>OGG Video</>);
        
        // Audio Files
        case "mp3":
            return (<>MP3 Audio</>);
        case "wav":
            return (<>WAV Audio</>);
        case "ogg":
            return (<>OGG Audio</>);
        case "flac":
            return (<>FLAC Audio</>);
        case "aac":
            return (<>AAC Audio</>);
        case "m4a":
            return (<>M4A Audio</>);
        case "wma":
            return (<>Windows Media Audio</>);
        case "aiff":
            return (<>AIFF Audio</>);
        case "opus":
            return (<>Opus Audio</>);
        case "mid":
        case "midi":
            return (<>MIDI Audio</>);
        
        // Web & Frontend Development
        case "html":
        case "htm":
        case "xhtml":
            return (<>HTML Document</>);
        case "css":
            return (<>CSS Stylesheet</>);
        case "scss":
        case "sass":
            return (<>Sass Stylesheet</>);
        case "less":
            return (<>Less Stylesheet</>);
        case "js":
            return (<>JavaScript File</>);
        case "jsx":
            return (<>React JSX File</>);
        case "ts":
            return (<>TypeScript File</>);
        case "tsx":
            return (<>React TSX File</>);
        case "json":
            return (<>JSON File</>);
        case "jsonc":
        case "json5":
            return (<>JSON with Comments</>);
        case "php":
            return (<>PHP File</>);
        case "asp":
        case "aspx":
            return (<>ASP.NET File</>);
        case "jsp":
            return (<>JSP File</>);
        
        // Programming Languages
        case "py":
            return (<>Python File</>);
        case "java":
            return (<>Java File</>);
        case "class":
            return (<>Java Class File</>);
        case "jar":
            return (<>Java Archive</>);
        case "c":
            return (<>C File</>);
        case "cpp":
        case "cc":
        case "cxx":
            return (<>C++ File</>);
        case "h":
        case "hpp":
            return (<>C/C++ Header</>);
        case "cs":
            return (<>C# File</>);
        case "go":
            return (<>Go File</>);
        case "rs":
            return (<>Rust File</>);
        case "rb":
            return (<>Ruby File</>);
        case "swift":
            return (<>Swift File</>);
        case "kt":
        case "kts":
            return (<>Kotlin File</>);
        
        // Archives
        case "zip":
            return (<>ZIP Archive</>);
        case "rar":
            return (<>RAR Archive</>);
        case "7z":
            return (<>7-Zip Archive</>);
        case "tar":
            return (<>TAR Archive</>);
        case "gz":
        case "gzip":
            return (<>GZip Archive</>);
        case "bz2":
        case "bzip2":
            return (<>BZip2 Archive</>);
        case "xz":
            return (<>XZ Archive</>);
        case "tgz":
            return (<>Compressed TAR</>);
        case "zst":
            return (<>Zstandard Archive</>);
        
        // Text & Config Files
        case "txt":
            return (<>Text File</>);
        case "md":
        case "markdown":
            return (<>Markdown Document</>);
        case "rtf":
            return (<>Rich Text Format</>);
        case "csv":
            return (<>CSV Spreadsheet</>);
        case "xml":
            return (<>XML Document</>);
        case "yaml":
        case "yml":
            return (<>YAML File</>);
        case "toml":
            return (<>TOML File</>);
        case "ini":
        case "cfg":
        case "conf":
            return (<>Configuration File</>);
        case "log":
            return (<>Log File</>);
        
        // Executables
        case "exe":
        case "com":
            return (<>Windows Executable</>);
        case "msi":
            return (<>Windows Installer</>);
        case "app":
            return (<>macOS Application</>);
        case "dmg":
            return (<>macOS Disk Image</>);
        case "pkg":
            return (<>macOS Package</>);
        case "deb":
            return (<>Debian Package</>);
        case "rpm":
            return (<>RPM Package</>);
        case "appimage":
            return (<>AppImage</>);
        case "apk":
            return (<>Android Package</>);
        case "sh":
        case "bash":
            return (<>Shell Script</>);
        case "bat":
        case "cmd":
            return (<>Batch File</>);
        case "ps1":
            return (<>PowerShell Script</>);
        
        // Database Files
        case "sql":
            return (<>SQL Script</>);
        case "sqlite":
        case "db":
            return (<>SQLite Database</>);
        case "mdb":
        case "accdb":
            return (<>Access Database</>);
        
        // Font Files
        case "ttf":
            return (<>TrueType Font</>);
        case "otf":
            return (<>OpenType Font</>);
        case "woff":
            return (<>Web Font</>);
        case "woff2":
            return (<>Web Font 2.0</>);
        case "eot":
            return (<>Embedded Font</>);
        
        // 3D & Design Files
        case "obj":
            return (<>3D Object</>);
        case "fbx":
            return (<>FBX 3D Model</>);
        case "glb":
        case "gltf":
            return (<>glTF 3D Model</>);
        
        // Binary & System Files
        case "dll":
            return (<>Windows Library</>);
        case "so":
            return (<>Shared Object</>);
        case "dylib":
            return (<>macOS Library</>);
        case "iso":
            return (<>Disk Image</>);
        case "dat":
        case "bin":
            return (<>Binary Data</>);
            
        // Default case for all other file types
        default:
            return (<>File</>);
    }
}
