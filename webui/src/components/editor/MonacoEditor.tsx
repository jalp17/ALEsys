import { useRef, useCallback } from 'react';
import Editor, { OnMount, OnChange } from '@monaco-editor/react';

interface MonacoEditorProps {
  value: string;
  language: string;
  onChange?: (value: string) => void;
  onSave?: (value: string) => void;
  readOnly?: boolean;
  height?: string;
}

export function MonacoEditor({
  value,
  language,
  onChange,
  onSave,
  readOnly = false,
  height = '100%',
}: MonacoEditorProps) {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const editorRef = useRef<any>(null);

  const handleMount: OnMount = useCallback(
    (editor, monaco) => {
      editorRef.current = editor;

      // Define custom dark theme
      monaco.editor.defineTheme('alesys-dark', {
        base: 'vs-dark',
        inherit: true,
        rules: [],
        colors: {
          'editor.background': '#1a1b26',
          'editor.foreground': '#a9b1d6',
          'editor.lineHighlightBackground': '#1e2030',
          'editor.selectionBackground': '#33467c',
          'editorCursor.foreground': '#c0caf5',
          'editorLineNumber.foreground': '#565f89',
          'editorLineNumber.activeForeground': '#c0caf5',
        },
      });
      monaco.editor.setTheme('alesys-dark');

      // Ctrl+S / Cmd+S to save
      editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS, () => {
        const currentValue = editor.getValue();
        onSave?.(currentValue);
      });
    },
    [onSave]
  );

  const handleChange: OnChange = useCallback(
    (val) => {
      if (val !== undefined) {
        onChange?.(val);
      }
    },
    [onChange]
  );

  return (
    <Editor
      height={height}
      language={language}
      value={value}
      theme="alesys-dark"
      onChange={handleChange}
      onMount={handleMount}
      options={{
        readOnly,
        fontSize: 14,
        fontFamily: "'JetBrains Mono', 'Fira Code', monospace",
        minimap: { enabled: false },
        scrollBeyondLastLine: false,
        wordWrap: 'on',
        automaticLayout: true,
        tabSize: 4,
        renderWhitespace: 'selection',
        bracketPairColorization: { enabled: true },
        cursorBlinking: 'smooth',
        cursorSmoothCaretAnimation: 'on',
        smoothScrolling: true,
        padding: { top: 12 },
      }}
    />
  );
}
