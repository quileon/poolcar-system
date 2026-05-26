import 'package:flutter/material.dart';
import 'package:mobile/src/rust/api/history.dart';
import 'package:shared_preferences/shared_preferences.dart';

class HistoryPage extends StatefulWidget {
  const HistoryPage({super.key});

  @override
  State<HistoryPage> createState() => _HistoryPageState();
}

class _HistoryPageState extends State<HistoryPage> {
  List<CarStatus>? _historyItems;
  bool _isLoading = true;
  String? _errorMessage;

  @override
  void initState() {
    super.initState();
    _loadHistory();
  }

  Future<void> _loadHistory() async {
    setState(() {
      _isLoading = true;
      _errorMessage = null;
    });

    final prefs = await SharedPreferences.getInstance();
    final token = prefs.getString('auth_token');

    if (token == null || token.isEmpty) {
      if (!mounted) return;
      Navigator.of(context).pushReplacementNamed('/login');
      return;
    }

    try {
      final result = await getHistories(token: token);
      
      if (!mounted) return;
      
      if (result.success) {
        setState(() {
          _historyItems = result.items ?? [];
          _isLoading = false;
        });
      } else {
        setState(() {
          _errorMessage = result.errorMessage ?? 'Failed to load history';
          _isLoading = false;
        });
        
        // If authentication failed, redirect to login
        if (_errorMessage!.toLowerCase().contains('unauthorized') || 
            _errorMessage!.toLowerCase().contains('token')) {
          await prefs.remove('auth_token');
          if (!mounted) return;
          Navigator.of(context).pushReplacementNamed('/login');
        }
      }
    } catch (e) {
      if (!mounted) return;
      setState(() {
        _errorMessage = e.toString();
        _isLoading = false;
      });
    }
  }

  Future<void> _logout() async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.remove('auth_token');
    if (!mounted) return;
    Navigator.of(context).pushReplacementNamed('/login');
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('History Dashboard'),
        actions: [
          IconButton(
            icon: const Icon(Icons.logout),
            onPressed: _logout,
          ),
        ],
      ),
      body: _isLoading 
          ? const Center(child: CircularProgressIndicator())
          : _errorMessage != null
              ? Center(
                  child: Column(
                    mainAxisAlignment: MainAxisAlignment.center,
                    children: [
                      Text('Error: $_errorMessage', style: const TextStyle(color: Colors.red)),
                      const SizedBox(height: 16),
                      ElevatedButton(
                        onPressed: _loadHistory,
                        child: const Text('Retry'),
                      )
                    ],
                  ),
                )
              : _historyItems == null || _historyItems!.isEmpty
                  ? const Center(child: Text('No history items found.'))
                  : RefreshIndicator(
                      onRefresh: _loadHistory,
                      child: ListView.builder(
                        itemCount: _historyItems!.length,
                        itemBuilder: (context, index) {
                          final item = _historyItems![index];
                          return ListTile(
                            leading: CircleAvatar(child: Text(item.carId.toString())),
                            title: Text('${item.carName} (${item.carPoliceNumber})'),
                            subtitle: Text('${item.statusType} at ${item.recordedAt}'),
                            trailing: Column(
                              mainAxisAlignment: MainAxisAlignment.center,
                              crossAxisAlignment: CrossAxisAlignment.end,
                              children: [
                                Text('${item.kilometres.toStringAsFixed(1)} km'),
                                Text('Gas: ${(item.gasLevel * 100).toStringAsFixed(0)}%'),
                              ],
                            ),
                          );
                        },
                      ),
                    ),
      floatingActionButton: FloatingActionButton(
        onPressed: () {
          Navigator.of(context).pushNamed('/camera');
        },
        tooltip: 'Scan QR Code',
        child: const Icon(Icons.qr_code_scanner),
      ),
    );
  }
}
