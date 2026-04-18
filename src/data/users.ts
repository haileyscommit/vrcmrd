import { LimitedUserInstance } from "vrchat";

export type Platform = 'pc' | 'android' | 'ios';
export type PerformanceRank = 'VeryPoor' | 'Poor' | 'Medium' | 'Good' | 'Excellent';

export type User = {
  id: string;
  username: string;
  avatarName: string;
  pronouns: string | null;
  status: string | null;
  perfRank?: PerformanceRank;
  accountCreated: number | null; // e.g. "3y"
  joinTime: number; // e.g. "13:12"
  leaveTime: number | null; // e.g. "13:24"
  advisories: ActiveAdvisory[];
  ageVerified: boolean;
  platform: Platform | null;
  recentlyKicked: boolean;
  trustRank?: TrustRank;
};

export type GetUserInfoResponse = {
  local: User | null;
  remote: LimitedUserInstance | null; // TODO: define remote user type
};

export function getHighestAdvisoryLevel(advisories: ActiveAdvisory[]): number {
  let highestLevel = 0;
  for (const advisory of advisories) {
    if (advisory.level > highestLevel) {
      highestLevel = advisory.level;
    }
    if (highestLevel === 4) {
      break;
    }
  }
  return highestLevel;
}

export type ActiveAdvisory = {
  id: string;
  message: string;
  level: 0 | 1 | 2 | 3 | 4;
  relevantGroupId?: string;
};

export type TrustRank = 'Nuisance' | 'Visitor' | 'NewUser' | 'User' | 'KnownUser' | 'TrustedUser' | 'Admin';

/** @deprecated Get actual users instead */
export const users: User[] = [
  {
    id: 'u1',
    username: 'NeonSparrow',
    avatarName: 'Neon Sparrow',
    perfRank: 'Excellent',
    pronouns: null,
    status: null,
    accountCreated: 4,
    joinTime: 7203,
    leaveTime: 7357,
    advisories: [],
    ageVerified: true,
    platform: 'pc',
    trustRank: 'TrustedUser',
    recentlyKicked: false,
  },
  {
    id: 'u2',
    username: 'PixelPanda',
    avatarName: 'Pixel Panda',
    perfRank: 'Good',
    pronouns: null,
    status: null,
    accountCreated: 1,
    joinTime: 7254,
    leaveTime: 7320,
    advisories: [],
    ageVerified: true,
    platform: 'android',
    trustRank: 'User',
    recentlyKicked: false,
  },
  {
    id: 'u3',
    username: 'Skyline',
    avatarName: 'Skyline',
    perfRank: 'Medium',
    pronouns: null,
    status: null,
    accountCreated: 5,
    joinTime: 7100,
    leaveTime: 7230,
    advisories: [],
    ageVerified: false,
    platform: 'ios',
    trustRank: 'NewUser',
    recentlyKicked: false,
  },
  {
    id: 'u4',
    username: 'Glitch',
    avatarName: 'Glitch',
    perfRank: 'VeryPoor',
    pronouns: null,
    status: null,
    accountCreated: 2,
    joinTime: 7475,
    leaveTime: 7555,
    advisories: [],
    ageVerified: false,
    platform: 'pc',
    trustRank: 'Nuisance',
    recentlyKicked: true,
  },
  {
    id: 'u5',
    username: 'Luma',
    avatarName: 'Luma',
    perfRank: 'Medium',
    pronouns: null,
    status: null,
    accountCreated: 0.75,
    joinTime: 7600,
    leaveTime: 7822,
    advisories: [],
    ageVerified: true,
    platform: 'ios',
    trustRank: 'KnownUser',
    recentlyKicked: false,
  }
];
