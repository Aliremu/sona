import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Button } from './ui/button';

export function NativeWindowTest() {
  const [result, setResult] = useState<string>('');

  const createVSTWindow = async () => {
    try {
      const response = await invoke('create_vst_window');
      setResult(`Success: ${response}`);
    } catch (error) {
      setResult(`Error: ${error}`);
    }
  };

  const openPluginEditor = async () => {
    try {
      const response = await invoke('open_plugin_editor');
      setResult(`Success: ${response}`);
    } catch (error) {
      setResult(`Error: ${error}`);
    }
  };

  return (
    <div className="p-4 space-y-4">
      <h2 className="text-xl font-bold">Native Window Test</h2>
      
      <div className="space-x-2">
        <Button onClick={createVSTWindow}>
          Create VST Window
        </Button>
        <Button onClick={openPluginEditor}>
          Open Plugin Editor
        </Button>
      </div>
      
      {result && (
        <div className="p-2 bg-gray-100 rounded">
          <pre>{result}</pre>
        </div>
      )}
    </div>
  );
}
