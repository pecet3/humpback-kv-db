import AceEditor from "react-ace";
import "ace-builds/src-noconflict/mode-javascript";
import "ace-builds/src-noconflict/theme-monokai";
import "ace-builds/src-noconflict/ext-language_tools";
import { useEffect, useRef, useState } from "react";

export const ExecuteCodeForm: React.FC<{
  onExecute: (code: string) => void;
  resultsResponses: string[];
  isLoading: boolean;
}> = ({ onExecute, isLoading, resultsResponses: results }) => {
  const [code, setCode] = useState(localStorage.getItem("code") || "");
  //   const [results, setResults] = useState<string[]>([]);
  //   useEffect(() => {
  //     console.log(resultsResponses);
  //     setResults(resultsResponses);
  //   }, [resultsResponses]);
  const editorRef = useRef<any>(null);

  useEffect(() => {
    localStorage.setItem("code", code);
  }, [code]);
  const handleSubmit = () => {
    if (code.trim()) {
      onExecute(code);
    }
  };
  const handleEditorLoad = (editor: any) => {
    editorRef.current = editor;

    editor.commands.addCommand({
      name: "InsertKVGet",

      bindKey: { win: "Ctrl-K", mac: "Command-K" },
      exec: () => {
        editor.insert(`kv.get("key")`);
      },
    });

    editor.commands.addCommand({
      name: "InsertFileRead",
      bindKey: { win: "Ctrl-R", mac: "Command-R" },
      exec: () => {
        editor.insert(`file.read("path/to/file.txt")`);
      },
    });

    editor.commands.addCommand({
      name: "InsertHttpGet",
      bindKey: { win: "Ctrl-H", mac: "Command-H" },
      exec: () => {
        editor.insert(`http.get("https://example.com")`);
      },
    });
  };
  return (
    <div className="w-full bg-slate-900 py-2 pb-12 m-auto rounded-xl h-full flex flex-col justify-center items-center gap-4">
      <AceEditor
        height="400px"
        width="600px"
        value={code}
        mode="javascript"
        theme="monokai"
        onLoad={handleEditorLoad}
        onChange={(v) => setCode(v)}
        fontSize="16px"
        highlightActiveLine={true}
        setOptions={{
          enableLiveAutocompletion: true,
          showLineNumbers: true,
          tabSize: 4,
        }}
        editorProps={{ $blockScrolling: true }}
      />
      <div className="h-32 overflow-y-auto">
        {results.map((r, idx) => (
          <p key={idx} className="font-mono text-xs my-2 text-white">
            {r}
          </p>
        ))}
      </div>
      <button
        type="button"
        onClick={handleSubmit}
        disabled={isLoading}
        className="w-32 bg-slate-500  hover:bg-slate-400  font-mono font-bold hover:cursor-pointer disabled:bg-gray-400 text-white py-2 px-4 rounded-lg transition-colors"
      >
        {isLoading ? "Executing..." : "Execute"}
      </button>
    </div>
  );
};
