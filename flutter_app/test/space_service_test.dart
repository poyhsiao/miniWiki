import 'package:flutter_test/flutter_test.dart';
import 'package:miniwiki/domain/entities/space.dart';
import 'package:miniwiki/domain/entities/space_membership.dart';
import 'package:miniwiki/services/space_service.dart';
import 'package:miniwiki/domain/repositories/space_repository.dart';
import 'package:miniwiki/core/network/network_error.dart';
import 'package:mocktail/mocktail.dart';

class MockSpaceRepository extends Mock implements SpaceRepository {}

void main() {
  group('SpaceService', () {
    late SpaceService spaceService;
    late MockSpaceRepository mockRepository;

    setUp(() {
      mockRepository = MockSpaceRepository();
      spaceService = SpaceService(spaceRepository: mockRepository);
    });

    group('listSpaces', () {
      test('returns list of spaces for authenticated user', () async {
        final spaces = [
          Space(
            id: 'space-1',
            name: 'Personal Wiki',
            ownerId: 'user-1',
            createdAt: DateTime.now(),
            updatedAt: DateTime.now(),
          ),
          Space(
            id: 'space-2',
            name: 'Work Notes',
            ownerId: 'user-1',
            createdAt: DateTime.now(),
            updatedAt: DateTime.now(),
          ),
        ];

        when(() => mockRepository.listSpaces()).thenAnswer((_) async => spaces);

        final result = await spaceService.listSpaces();

        expect(result.length, 2);
        expect(result[0].name, 'Personal Wiki');
        expect(result[1].name, 'Work Notes');
        verify(() => mockRepository.listSpaces()).called(1);
      });

      test('returns empty list when user has no spaces', () async {
        when(() => mockRepository.listSpaces()).thenAnswer((_) async => []);

        final result = await spaceService.listSpaces();

        expect(result.isEmpty, true);
      });

      test('throws exception when repository fails', () async {
        when(() => mockRepository.listSpaces())
            .thenThrow(NetworkError('Failed to fetch spaces', 500));

        expect(() => spaceService.listSpaces(), throwsA(isA<NetworkError>()));
      });
    });

    group('getSpace', () {
      test('returns space when it exists', () async {
        final space = Space(
          id: 'space-1',
          name: 'Personal Wiki',
          ownerId: 'user-1',
          createdAt: DateTime.now(),
          updatedAt: DateTime.now(),
        );

        when(() => mockRepository.getSpace('space-1')).thenAnswer((_) async => space);

        final result = await spaceService.getSpace('space-1');

        expect(result.id, 'space-1');
        expect(result.name, 'Personal Wiki');
      });

      test('throws exception when space not found', () async {
        when(() => mockRepository.getSpace('non-existent'))
            .thenThrow(NetworkError('Space not found', 404));

        expect(
          () => spaceService.getSpace('non-existent'),
          throwsA(isA<NetworkError>()),
        );
      });
    });

    group('createSpace', () {
      test('creates a new space successfully', () async {
        final newSpace = Space(
          id: 'space-new',
          name: 'New Space',
          ownerId: 'user-1',
          createdAt: DateTime.now(),
          updatedAt: DateTime.now(),
        );

        when(() => mockRepository.createSpace(
          name: 'New Space',
          icon: null,
          description: null,
          isPublic: false,
        )).thenAnswer((_) async => newSpace);

        final result = await spaceService.createSpace(
          name: 'New Space',
          icon: null,
          description: null,
          isPublic: false,
        );

        expect(result.name, 'New Space');
        expect(result.ownerId, 'user-1');
      });

      test('throws exception for empty name', () async {
        expect(
          () => spaceService.createSpace(
            name: '',
            icon: null,
            description: null,
            isPublic: false,
          ),
          throwsArgumentError,
        );
      });

      test('throws exception for name exceeding 200 characters', () async {
        final longName = 'a' * 201;

        expect(
          () => spaceService.createSpace(
            name: longName,
            icon: null,
            description: null,
            isPublic: false,
          ),
          throwsArgumentError,
        );
      });
    });

    group('updateSpace', () {
      test('updates space successfully', () async {
        final updatedSpace = Space(
          id: 'space-1',
          name: 'Updated Name',
          icon: '🚀',
          description: 'Updated description',
          ownerId: 'user-1',
          createdAt: DateTime.now(),
          updatedAt: DateTime.now(),
        );

        when(() => mockRepository.updateSpace(
          'space-1',
          name: 'Updated Name',
          icon: '🚀',
          description: 'Updated description',
          isPublic: null,
        )).thenAnswer((_) async => updatedSpace);

        final result = await spaceService.updateSpace(
          'space-1',
          name: 'Updated Name',
          icon: '🚀',
          description: 'Updated description',
        );

        expect(result.name, 'Updated Name');
        expect(result.icon, '🚀');
      });
    });

    group('deleteSpace', () {
      test('deletes space successfully', () async {
        when(() => mockRepository.deleteSpace('space-1')).thenAnswer((_) async => {});

        await spaceService.deleteSpace('space-1');

        verify(() => mockRepository.deleteSpace('space-1')).called(1);
      });

      test('throws exception when deletion fails', () async {
        when(() => mockRepository.deleteSpace('space-1'))
            .thenThrow(NetworkError('Failed to delete', 500));

        expect(
          () => spaceService.deleteSpace('space-1'),
          throwsA(isA<NetworkError>()),
        );
      });
    });

    group('listMembers', () {
      test('returns list of members for space', () async {
        final members = [
          SpaceMembership(
            id: 'member-1',
            spaceId: 'space-1',
            userId: 'user-1',
            role: 'owner',
            joinedAt: DateTime.now(),
            invitedBy: 'user-1',
          ),
          SpaceMembership(
            id: 'member-2',
            spaceId: 'space-1',
            userId: 'user-2',
            role: 'editor',
            joinedAt: DateTime.now(),
            invitedBy: 'user-1',
          ),
        ];

        when(() => mockRepository.listMembers('space-1')).thenAnswer((_) async => members);

        final result = await spaceService.listMembers('space-1');

        expect(result.length, 2);
        expect(result[0].role, 'owner');
        expect(result[1].role, 'editor');
      });
    });

    group('addMember', () {
      test('adds member to space successfully', () async {
        final membership = SpaceMembership(
          id: 'member-new',
          spaceId: 'space-1',
          userId: 'user-2',
          role: 'editor',
          joinedAt: DateTime.now(),
          invitedBy: 'user-1',
        );

        when(() => mockRepository.addMember(
          'space-1',
          'user-2',
          'editor',
        )).thenAnswer((_) async => membership);

        final result = await spaceService.addMember(
          'space-1',
          'user-2',
          'editor',
        );

        expect(result.userId, 'user-2');
        expect(result.role, 'editor');
      });

      test('throws exception for invalid role', () async {
        expect(
          () => spaceService.addMember('space-1', 'user-2', 'invalid_role'),
          throwsArgumentError,
        );
      });
    });

    group('updateMemberRole', () {
      test('updates member role successfully', () async {
        final updatedMembership = SpaceMembership(
          id: 'member-1',
          spaceId: 'space-1',
          userId: 'user-2',
          role: 'viewer',
          joinedAt: DateTime.now(),
          invitedBy: 'user-1',
        );

        when(() => mockRepository.updateMemberRole('space-1', 'user-2', 'viewer'))
            .thenAnswer((_) async => updatedMembership);

        final result = await spaceService.updateMemberRole(
          'space-1',
          'user-2',
          'viewer',
        );

        expect(result.role, 'viewer');
      });

      test('throws exception when trying to demote owner', () async {
        when(() => mockRepository.updateMemberRole('space-1', 'user-1', 'viewer'))
            .thenThrow(NetworkError('Cannot change owner role', 400));

        expect(
          () => spaceService.updateMemberRole('space-1', 'user-1', 'viewer'),
          throwsA(isA<NetworkError>()),
        );
      });
    });

    group('removeMember', () {
      test('removes member from space successfully', () async {
        when(() => mockRepository.removeMember('space-1', 'user-2'))
            .thenAnswer((_) async => {});

        await spaceService.removeMember('space-1', 'user-2');

        verify(() => mockRepository.removeMember('space-1', 'user-2')).called(1);
      });

      test('throws exception when trying to remove owner', () async {
        when(() => mockRepository.removeMember('space-1', 'user-1'))
            .thenThrow(NetworkError('Cannot remove owner', 400));

        expect(
          () => spaceService.removeMember('space-1', 'user-1'),
          throwsA(isA<NetworkError>()),
        );
      });
    });
  });
}
