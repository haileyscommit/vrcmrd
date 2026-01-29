export type Platform = 'pc' | 'android' | 'ios';
export type PerformanceRank = 'VeryPoor' | 'Poor' | 'Medium' | 'Good' | 'Excellent';

export type User = {
  id: string;
  username: string;
  avatarName: string;
  perfRank: PerformanceRank;
  accountAge: string; // e.g. "3y"
  joinTime: number; // e.g. "13:12"
  leaveTime: number | null; // e.g. "13:24"
  advisories: boolean;
  ageVerified: boolean;
  platform: Platform | null;
};

/** @deprecated Get actual users instead */
export const users: User[] = [
  {
    id: 'u1',
    username: 'NeonSparrow',
    avatarName: 'Neon Sparrow',
    perfRank: 'Excellent',
    accountAge: '4y',
    joinTime: 7203,
    leaveTime: 7357,
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
    joinTime: 7254,
    leaveTime: 7320,
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
    joinTime: 7100,
    leaveTime: 7230,
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
    joinTime: 7475,
    leaveTime: 7555,
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
    joinTime: 7600,
    leaveTime: 7822,
    advisories: false,
    ageVerified: true,
    platform: 'ios'
  }
];
