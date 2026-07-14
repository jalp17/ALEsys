export function Settings() {
  return (
    <div className="max-w-2xl mx-auto">
      <h1 className="text-2xl font-bold mb-6">Configuración</h1>
      
      <div className="space-y-6">
        <div className="bg-dark-800 rounded-lg p-6">
          <h2 className="text-lg font-semibold mb-4">Modelo LLM</h2>
          <div className="space-y-3">
            <div>
              <label className="block text-sm text-gray-400 mb-1">Backend</label>
              <select className="w-full px-3 py-2 bg-dark-900 border border-gray-700 rounded text-white">
                <option>MistralRS (GGUF)</option>
                <option>ONNX Runtime</option>
              </select>
            </div>
            <div>
              <label className="block text-sm text-gray-400 mb-1">Modelo</label>
              <input 
                type="text" 
                defaultValue="google/gemma-2b-it"
                className="w-full px-3 py-2 bg-dark-900 border border-gray-700 rounded text-white"
              />
            </div>
          </div>
        </div>

        <div className="bg-dark-800 rounded-lg p-6">
          <h2 className="text-lg font-semibold mb-4">Base de Datos</h2>
          <div className="space-y-3">
            <div>
              <label className="block text-sm text-gray-400 mb-1">Connection String</label>
              <input 
                type="text" 
                defaultValue="postgresql://alesys:***@localhost:5432/alesys"
                className="w-full px-3 py-2 bg-dark-900 border border-gray-700 rounded text-white font-mono text-sm"
              />
            </div>
          </div>
        </div>

        <div className="bg-dark-800 rounded-lg p-6">
          <h2 className="text-lg font-semibold mb-4">Embeddings</h2>
          <div className="space-y-3">
            <div>
              <label className="block text-sm text-gray-400 mb-1">Modelo</label>
              <input 
                type="text" 
                defaultValue="sentence-transformers/all-MiniLM-L6-v2"
                className="w-full px-3 py-2 bg-dark-900 border border-gray-700 rounded text-white"
              />
            </div>
            <div>
              <label className="block text-sm text-gray-400 mb-1">Dimensión</label>
              <input 
                type="number" 
                defaultValue={384}
                className="w-full px-3 py-2 bg-dark-900 border border-gray-700 rounded text-white"
              />
            </div>
          </div>
        </div>

        <div className="flex justify-end gap-3">
          <button className="px-4 py-2 text-gray-400 hover:text-white transition">
            Cancelar
          </button>
          <button className="px-6 py-2 bg-primary-600 rounded hover:bg-primary-700 transition font-semibold">
            Guardar
          </button>
        </div>
      </div>
    </div>
  );
}