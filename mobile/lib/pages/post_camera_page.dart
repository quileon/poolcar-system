import 'package:flutter/material.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:mobile/src/rust/api/car.dart';
import 'package:mobile/src/rust/api/history.dart';

class PostCameraPage extends StatefulWidget {
  const PostCameraPage({super.key});

  @override
  State<PostCameraPage> createState() => _PostCameraPageState();
}

class _PostCameraPageState extends State<PostCameraPage> {
  String? _scannedData;
  Car? _car;
  bool _isLoading = true;
  String? _errorMessage;

  final _formKey = GlobalKey<FormState>();
  final _gasLevelController = TextEditingController();
  final _kilometresController = TextEditingController();
  String _statusType = 'Departure'; // Default to Departure
  bool _isSubmitting = false;

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    if (_scannedData == null) {
      _scannedData = ModalRoute.of(context)?.settings.arguments as String?;
      _fetchCarDetails();
    }
  }

  Future<void> _fetchCarDetails() async {
    setState(() {
      _isLoading = true;
      _errorMessage = null;
    });

    if (_scannedData == null || _scannedData!.isEmpty) {
      setState(() {
        _errorMessage = 'Invalid QR Code: No data scanned.';
        _isLoading = false;
      });
      return;
    }

    final carId = int.tryParse(_scannedData!);
    if (carId == null) {
      setState(() {
        _errorMessage = 'Invalid QR Code: Expected a numeric Car ID, but got "$_scannedData".';
        _isLoading = false;
      });
      return;
    }

    final prefs = await SharedPreferences.getInstance();
    final token = prefs.getString('auth_token');

    if (token == null || token.isEmpty) {
      if (!mounted) return;
      Navigator.of(context).pushReplacementNamed('/login');
      return;
    }

    try {
      final result = await getCarById(token: token, carId: carId);
      
      if (!mounted) return;

      if (result.success && result.car != null) {
        setState(() {
          _car = result.car;
          _isLoading = false;
        });
      } else {
        setState(() {
          _errorMessage = result.errorMessage ?? 'Car not found or you do not have permission.';
          _isLoading = false;
        });
      }
    } catch (e) {
      if (!mounted) return;
      setState(() {
        _errorMessage = 'Error fetching car details: $e';
        _isLoading = false;
      });
    }
  }

  Future<void> _submitHistory() async {
    if (!_formKey.currentState!.validate()) return;

    setState(() {
      _isSubmitting = true;
      _errorMessage = null;
    });

    final prefs = await SharedPreferences.getInstance();
    final token = prefs.getString('auth_token');

    if (token == null || token.isEmpty) {
      if (!mounted) return;
      Navigator.of(context).pushReplacementNamed('/login');
      return;
    }

    final gasLevel = double.tryParse(_gasLevelController.text) ?? 0.0;
    final kilometres = double.tryParse(_kilometresController.text) ?? 0.0;

    try {
      final result = await postHistory(
        token: token,
        carId: _car!.carId,
        gasLevel: gasLevel, // API expects raw gas level in bars
        kilometres: kilometres,
        statusType: _statusType,
      );

      if (!mounted) return;

      if (result.success) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(content: Text('Car status successfully recorded!')),
        );
        Navigator.of(context).pushReplacementNamed('/history');
      } else {
        setState(() {
          _errorMessage = result.errorMessage ?? 'Failed to record car status.';
        });
      }
    } catch (e) {
      if (!mounted) return;
      setState(() {
        _errorMessage = 'Submission error: $e';
      });
    } finally {
      if (mounted) {
        setState(() {
          _isSubmitting = false;
        });
      }
    }
  }

  @override
  void dispose() {
    _gasLevelController.dispose();
    _kilometresController.dispose();
    super.dispose();
  }

  Widget _buildErrorState() {
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(24.0),
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            const Icon(Icons.error_outline, size: 80, color: Colors.red),
            const SizedBox(height: 24),
            Text(
              _errorMessage ?? 'Unknown error',
              style: const TextStyle(fontSize: 18, color: Colors.red),
              textAlign: TextAlign.center,
            ),
            const SizedBox(height: 48),
            ElevatedButton.icon(
              onPressed: () => Navigator.of(context).pop(),
              icon: const Icon(Icons.arrow_back),
              label: const Text('Back to Dashboard'),
              style: ElevatedButton.styleFrom(
                minimumSize: const Size(double.infinity, 50),
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildForm() {
    return Form(
      key: _formKey,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          const Text(
            'Record Status',
            style: TextStyle(fontSize: 20, fontWeight: FontWeight.bold),
          ),
          const SizedBox(height: 16),
          DropdownButtonFormField<String>(
            initialValue: _statusType,
            decoration: const InputDecoration(
              labelText: 'Status Type',
              border: OutlineInputBorder(),
            ),
            items: const [
              DropdownMenuItem(value: 'Departure', child: Text('Departure')),
              DropdownMenuItem(value: 'Return', child: Text('Return')),
            ],
            onChanged: (value) {
              setState(() {
                if (value != null) _statusType = value;
              });
            },
          ),
          const SizedBox(height: 16),
          TextFormField(
            controller: _gasLevelController,
            decoration: const InputDecoration(
              labelText: 'Gas Bar Level',
              hintText: '2.5',
              border: OutlineInputBorder(),
              suffixText: 'bar',
            ),
            keyboardType: const TextInputType.numberWithOptions(decimal: true),
            validator: (value) {
              if (value == null || value.isEmpty) return 'Please enter gas bar level';
              final numValue = double.tryParse(value);
              if (numValue == null) return 'Enter a valid number';
              if (numValue < 0) return 'Gas bar level cannot be negative';
              return null;
            },
          ),
          const SizedBox(height: 16),
          TextFormField(
            controller: _kilometresController,
            decoration: const InputDecoration(
              labelText: 'Odometer',
              hintText: '727',
              border: OutlineInputBorder(),
              suffixText: 'km',
            ),
            keyboardType: const TextInputType.numberWithOptions(decimal: true),
            validator: (value) {
              if (value == null || value.isEmpty) return 'Please enter odometer';
              final numValue = double.tryParse(value);
              if (numValue == null) return 'Enter a valid number';
              if (numValue < 0) return 'Odometer cannot be negative';
              return null;
            },
          ),
          if (_errorMessage != null) ...[
            const SizedBox(height: 16),
            Text(_errorMessage!, style: const TextStyle(color: Colors.red)),
          ],
          const SizedBox(height: 24),
          _isSubmitting
              ? const Center(child: CircularProgressIndicator())
              : ElevatedButton.icon(
                  onPressed: _submitHistory,
                  icon: const Icon(Icons.save),
                  label: const Text('Submit Status'),
                  style: ElevatedButton.styleFrom(
                    minimumSize: const Size(double.infinity, 50),
                  ),
                ),
        ],
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Car Details')),
      body: _isLoading
          ? const Center(child: CircularProgressIndicator())
          : _car == null
              ? _buildErrorState()
              : SingleChildScrollView(
                  padding: const EdgeInsets.all(24.0),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Card(
                        elevation: 4,
                        margin: EdgeInsets.zero,
                        child: Padding(
                          padding: const EdgeInsets.all(16.0),
                          child: Column(
                            crossAxisAlignment: CrossAxisAlignment.start,
                            children: [
                              Row(
                                children: [
                                  const Icon(Icons.directions_car, size: 40, color: Colors.blue),
                                  const SizedBox(width: 16),
                                  Expanded(
                                    child: Column(
                                      crossAxisAlignment: CrossAxisAlignment.start,
                                      children: [
                                        Text(
                                          _car!.name,
                                          style: const TextStyle(fontSize: 22, fontWeight: FontWeight.bold),
                                        ),
                                        Text(
                                          _car!.policeNumber,
                                          style: TextStyle(fontSize: 18, color: Colors.grey.shade700),
                                        ),
                                      ],
                                    ),
                                  ),
                                  Container(
                                    padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
                                    decoration: BoxDecoration(
                                      color: _car!.active != 0 ? Colors.green.shade100 : Colors.red.shade100,
                                      borderRadius: BorderRadius.circular(16),
                                    ),
                                    child: Text(
                                      _car!.active != 0 ? 'Active' : 'Inactive',
                                      style: TextStyle(
                                        color: _car!.active != 0 ? Colors.green.shade800 : Colors.red.shade800,
                                        fontWeight: FontWeight.bold,
                                      ),
                                    ),
                                  ),
                                ],
                              ),
                              const Divider(height: 32),
                              Text('Type: ${_car!.carType}', style: const TextStyle(fontSize: 16)),
                              if (_car!.trackerId != null) ...[
                                const SizedBox(height: 8),
                                Text('Tracker ID: ${_car!.trackerId}', style: const TextStyle(fontSize: 16)),
                              ]
                            ],
                          ),
                        ),
                      ),
                      const SizedBox(height: 32),
                      _buildForm(),
                    ],
                  ),
                ),
    );
  }
}
