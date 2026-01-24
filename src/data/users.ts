export type Platform = 'pc' | 'android' | 'ios';
export type PerformanceRank = 'VeryPoor' | 'Poor' | 'Medium' | 'Good' | 'Excellent';

export type User = {
  id: string;
  username: string;
  avatarName: string;
  perfRank: PerformanceRank;
  accountAge: string; // e.g. "3y"
  joinTime: string; // e.g. "13:12"
  leaveTime: string; // e.g. "13:24"
  advisories: boolean;
  ageVerified: boolean;
  platform: Platform;
};

export const users: User[] = [
  {
    id: 'u1',
    username: 'NeonSparrow',
    avatarName: 'Neon Sparrow',
    perfRank: 'Excellent',
    accountAge: '4y',
    joinTime: '12:03',
    leaveTime: '12:37',
    advisories: true,
    ageVerified: true,
    platform: 'pc'
  },
  {
    id: 'u2',
    username: 'PixelPanda',
    avatarName: 'Pixel Panda',
    perfRank: 'Good',
    accountAge: '1y',
    joinTime: '12:09',
    leaveTime: '12:20',
    advisories: false,
    ageVerified: true,
    platform: 'android'
  },
  {
    id: 'u3',
    username: 'Skyline',
    avatarName: 'Skyline',
    perfRank: 'Medium',
    accountAge: '6mo',
    joinTime: '11:50',
    leaveTime: '12:05',
    advisories: false,
    ageVerified: false,
    platform: 'ios'
  },
  {
    id: 'u4',
    username: 'Glitch',
    avatarName: 'Glitch',
    perfRank: 'VeryPoor',
    accountAge: '2y',
    joinTime: '12:25',
    leaveTime: '12:55',
    advisories: true,
    ageVerified: false,
    platform: 'pc'
  },
  {
    id: 'u5',
    username: 'Luma',
    avatarName: 'Luma',
    perfRank: 'Medium',
    accountAge: '9mo',
    joinTime: '12:40',
    leaveTime: '13:02',
    advisories: false,
    ageVerified: true,
    platform: 'ios'
  }
];
