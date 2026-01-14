class SpaceMembership {
  final String id;
  final String spaceId;
  final String userId;
  final String role;
  final DateTime joinedAt;
  final String invitedBy;

  const SpaceMembership({
    required this.id,
    required this.spaceId,
    required this.userId,
    required this.role,
    required this.joinedAt,
    required this.invitedBy,
  });

  factory SpaceMembership.fromJson(Map<String, dynamic> json) => SpaceMembership(
      id: json['id'] as String,
      spaceId: json['space_id'] as String,
      userId: json['user_id'] as String,
      role: json['role'] as String,
      joinedAt: DateTime.parse(json['joined_at'] as String),
      invitedBy: json['invited_by'] as String,
    );

  Map<String, dynamic> toJson() => {
      'id': id,
      'space_id': spaceId,
      'user_id': userId,
      'role': role,
      'joined_at': joinedAt.toIso8601String(),
      'invited_by': invitedBy,
    };

  @override
  bool operator ==(Object other) {
    if (identical(this, other)) return true;
    return other is SpaceMembership && other.id == id;
  }

  @override
  int get hashCode => id.hashCode;
}
