import 'package:file_picker/file_picker.dart';
import 'package:flutter/material.dart';
import 'package:fastshare_ui/src/rust/api/simple.dart';
import 'package:fastshare_ui/src/rust/frb_generated.dart';

Future<void> main() async {
  await RustLib.init();
  runApp(const MyApp());
}

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      debugShowCheckedModeBanner: false,
      theme: ThemeData.dark().copyWith(
        scaffoldBackgroundColor: const Color(0xFF1a1a2e),
        colorScheme: const ColorScheme.dark(
          primary: Color(0xFF4ECDC4),
          surface: Color(0xFF16213e),
        ),
      ),
      home: const FastShareHome(),
    );
  }
}

class FastShareHome extends StatefulWidget {
  const FastShareHome({super.key});

  @override
  State<FastShareHome> createState() => _FastShareHomeState();
}

class _FastShareHomeState extends State<FastShareHome> {
  String status = "Backend Idle";
  String? selectedFilePath;

  void _startBackend() async {
    setState(() {
      status = "Starting Backend...";
    });
    // This calls the Rust function we created via the bridge!
    final result = await startFastshare();
    setState(() {
      status = result;
    });
  }

  void _pickFile() async {
    FilePickerResult? result = await FilePicker.platform.pickFiles();
    if (result != null) {
      setState(() {
        selectedFilePath = result.files.single.path;
      });
      // Here we would call Rust to send the file
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text(
            'Selected: ${result.files.single.name}\nReady to send to a peer!',
          ),
        ),
      );
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: Center(
        child: SingleChildScrollView(
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              const Text(
                '⚡ FastShare',
                style: TextStyle(
                  fontSize: 48,
                  fontWeight: FontWeight.bold,
                  color: Color(0xFF4ECDC4),
                ),
              ),
              const SizedBox(height: 10),
              const Text(
                'Ultra-High-Performance P2P File Transfer',
                style: TextStyle(fontSize: 18, color: Colors.grey),
              ),
              const SizedBox(height: 50),
              Row(
                mainAxisAlignment: MainAxisAlignment.center,
                children: [
                  ElevatedButton.icon(
                    onPressed: _pickFile,
                    icon: const Icon(Icons.upload_file),
                    label: const Text('Send File'),
                    style: ElevatedButton.styleFrom(
                      padding: const EdgeInsets.symmetric(
                        horizontal: 24,
                        vertical: 16,
                      ),
                      backgroundColor: const Color(0xFF4ECDC4),
                      foregroundColor: const Color(0xFF1a1a2e),
                    ),
                  ),
                  const SizedBox(width: 16),
                  OutlinedButton.icon(
                    onPressed: () {},
                    icon: const Icon(Icons.download),
                    label: const Text('Receive File'),
                    style: OutlinedButton.styleFrom(
                      padding: const EdgeInsets.symmetric(
                        horizontal: 24,
                        vertical: 16,
                      ),
                      foregroundColor: const Color(0xFF4ECDC4),
                      side: const BorderSide(
                        color: Color(0xFF4ECDC4),
                        width: 2,
                      ),
                    ),
                  ),
                ],
              ),
              if (selectedFilePath != null)
                Padding(
                  padding: const EdgeInsets.all(16.0),
                  child: Text(
                    'Selected: $selectedFilePath',
                    style: const TextStyle(color: Colors.greenAccent),
                  ),
                ),
              const SizedBox(height: 50),
              Container(
                padding: const EdgeInsets.all(24),
                decoration: BoxDecoration(
                  color: Theme.of(context).colorScheme.surface,
                  borderRadius: BorderRadius.circular(16),
                ),
                child: Column(
                  children: [
                    const Text(
                      'System Status',
                      style: TextStyle(fontWeight: FontWeight.bold),
                    ),
                    const SizedBox(height: 10),
                    Text(status, textAlign: TextAlign.center),
                    const SizedBox(height: 20),
                    ElevatedButton(
                      onPressed: _startBackend,
                      child: const Text('Start Rust Engine (Discovery)'),
                    ),
                  ],
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
