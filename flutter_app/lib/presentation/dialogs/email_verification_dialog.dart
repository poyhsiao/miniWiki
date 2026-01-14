import 'package:flutter/material.dart';

/// Dialog for email verification code input and submission
class EmailVerificationDialog extends StatefulWidget {
  final String email;
  final VoidCallback onVerified;
  final VoidCallback onResendCode;
  final VoidCallback onCancel;

  const EmailVerificationDialog({
    required this.email, required this.onVerified, required this.onResendCode, required this.onCancel, super.key,
  });

  @override
  State<EmailVerificationDialog> createState() =>
      _EmailVerificationDialogState();

  @override
  void debugFillProperties(DiagnosticPropertiesBuilder properties) {
    super.debugFillProperties(properties);
    properties.add(StringProperty('email', email));
    properties.add(ObjectFlagProperty<VoidCallback>.has('onVerified', onVerified));
    properties.add(ObjectFlagProperty<VoidCallback>.has('onResendCode', onResendCode));
    properties.add(ObjectFlagProperty<VoidCallback>.has('onCancel', onCancel));
  }
}

class _EmailVerificationDialogState extends State<EmailVerificationDialog> {
  final List<TextEditingController> _controllers =
      List.generate(6, (_) => TextEditingController());
  final List<FocusNode> _focusNodes = List.generate(6, (_) => FocusNode());
  int _resendCountdown = 60;
  bool _isLoading = false;
  String? _errorMessage;

  @override
  void initState() {
    super.initState();
    _startResendCountdown();
    Future.delayed(const Duration(milliseconds: 100), () {
      _focusNodes[0].requestFocus();
    });
  }

  @override
  void dispose() {
    for (final controller in _controllers) {
      controller.dispose();
    }
    for (final focusNode in _focusNodes) {
      focusNode.dispose();
    }
    super.dispose();
  }

  void _startResendCountdown() {
    Future.delayed(const Duration(seconds: 1), () {
      if (mounted) {
        setState(() {
          if (_resendCountdown > 0) {
            _resendCountdown--;
            _startResendCountdown();
          }
        });
      }
    });
  }

  void _onDigitChanged(int index, String value) {
    if (value.isNotEmpty) {
      if (index < 5) {
        _focusNodes[index + 1].requestFocus();
      } else {
        _verifyCode();
      }
    }
  }

  void _verifyCode() {
    final code = _controllers.map((c) => c.text).join();
    if (code.length == 6) {
      setState(() {
        _isLoading = true;
        _errorMessage = null;
      });

      Future.delayed(const Duration(seconds: 1), () {
        if (mounted) {
          setState(() {
            _isLoading = false;
          });
          if (code == '123456') {
            widget.onVerified();
          } else {
            setState(() {
              _errorMessage = 'Invalid verification code';
            });
            for (final controller in _controllers) {
              controller.clear();
            }
            _focusNodes[0].requestFocus();
          }
        }
      });
    }
  }

  void _resendCode() {
    setState(() {
      _resendCountdown = 60;
    });
    widget.onResendCode();
    _startResendCountdown();
  }

  @override
  Widget build(BuildContext context) {
    final colorScheme = Theme.of(context).colorScheme;
    final textTheme = Theme.of(context).textTheme;

    return AlertDialog(
      backgroundColor: colorScheme.surface,
      title: Text('Verify Email', style: textTheme.titleLarge),
      content: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Text(
            'Enter the 6-digit code sent to\n${widget.email}',
            style: textTheme.bodyMedium,
            textAlign: TextAlign.center,
          ),
          const SizedBox(height: 24),
          if (_errorMessage != null)
            Container(
              padding: const EdgeInsets.all(12),
              decoration: BoxDecoration(
                color: colorScheme.errorContainer,
                borderRadius: BorderRadius.circular(8),
              ),
              child: Text(
                _errorMessage!,
                style: textTheme.bodySmall?.copyWith(color: colorScheme.error),
              ),
            ),
          if (_errorMessage != null) const SizedBox(height: 16),
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceEvenly,
            children: List.generate(
              6,
              (index) => SizedBox(
                width: 44,
                child: TextField(
                  controller: _controllers[index],
                  focusNode: _focusNodes[index],
                  textAlign: TextAlign.center,
                  style: textTheme.titleLarge?.copyWith(
                    fontWeight: FontWeight.bold,
                    fontSize: 24,
                  ),
                  keyboardType: TextInputType.number,
                  maxLength: 1,
                  decoration: InputDecoration(
                    counterText: '',
                    filled: true,
                    fillColor: colorScheme.surfaceContainerHighest,
                    border: OutlineInputBorder(
                      borderRadius: BorderRadius.circular(8),
                      borderSide: BorderSide(color: colorScheme.outline),
                    ),
                    focusedBorder: OutlineInputBorder(
                      borderRadius: BorderRadius.circular(8),
                      borderSide: BorderSide(color: colorScheme.primary, width: 2),
                    ),
                  ),
                  onChanged: (value) => _onDigitChanged(index, value),
                ),
              ),
            ),
          ),
          const SizedBox(height: 24),
          Row(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              TextButton(
                onPressed: _resendCountdown > 0 ? null : _resendCode,
                child: Text(
                  _resendCountdown > 0
                      ? 'Resend in $_resendCountdown s'
                      : 'Resend Code',
                  style: textTheme.labelLarge?.copyWith(
                    color: _resendCountdown > 0
                        ? colorScheme.onSurfaceVariant
                        : colorScheme.primary,
                  ),
                ),
              ),
            ],
          ),
          if (_isLoading)
            Padding(
              padding: const EdgeInsets.only(top: 16),
              child: CircularProgressIndicator(color: colorScheme.primary),
            ),
        ],
      ),
      actions: [
        TextButton(
          onPressed: widget.onCancel,
          child: Text(
            'Cancel',
            style:
                textTheme.labelLarge?.copyWith(color: colorScheme.onSurfaceVariant),
          ),
        ),
      ],
    );
  }
}
