import { Notice as NoticeType } from '@app/bindings/Notice';
import EyeOutlineIcon from 'mdi-preact/EyeOutlineIcon';
import AccountIcon from 'mdi-preact/AccountIcon';
import HexagonMultipleIcon from 'mdi-preact/HexagonMultipleIcon';
//import FlagIcon from 'mdi-preact/FlagIcon';
import { invoke } from '@tauri-apps/api/core';

export default function Notice({ notice }: { notice: NoticeType }) {
  const chipClasses = "flex flex-row items-center cursor-pointer gap-1 mr-2 px-2 py-1 bg-gray-200/40 dark:bg-gray-700/40 hover:bg-gray-300/60 dark:hover:bg-gray-600/60 border rounded-full text-xs";
  // function trim(str: string, maxLength: number) {
  //   if (str.length <= maxLength) return str;
  //   return str.slice(0, maxLength - 3) + '...';
  // }
  return (
    <div
      className={`m-2 border rounded-lg ${
        notice.level as any === 0
          ? 'bg-gray-50 dark:bg-gray-900 border-gray-400 dark:border-gray-600 text-gray-800 dark:text-gray-200'
          : notice.level as any === 1
          ? 'bg-blue-50 dark:bg-blue-900 border-blue-400 dark:border-blue-600 text-gray-800 dark:text-gray-200'
          : notice.level as any === 2
          ? 'bg-yellow-50 dark:bg-yellow-900 border-yellow-400 dark:border-yellow-600 text-gray-800 dark:text-gray-200'
          : notice.level as any === 3
          ? 'bg-orange-50 dark:bg-orange-900 border-orange-400 dark:border-orange-600 text-gray-800 dark:text-gray-200'
          : notice.level as any === 4
          ? 'bg-red-50 dark:bg-red-900 border-red-400 dark:border-red-600 text-gray-800 dark:text-gray-200'
          : 'bg-red-500 text-white'
      }`}
    >
      <div class="px-4 py-2">
        {notice.title && <h2 class="text-lg font-semibold mb-2">{notice.title}</h2>}
        <p class="mb-2">{notice.message}</p>
      </div>
      <div class={`rounded-b-lg p-4 flex flex-row gap-4 items-center text-sm ${notice.level as any === 0
          ? 'bg-gray-100 dark:bg-gray-800 text-gray-600 dark:text-gray-300'
          : notice.level as any === 1
          ? 'bg-blue-100 dark:bg-blue-800 text-gray-600 dark:text-gray-300'
          : notice.level as any === 2
          ? 'bg-yellow-100 dark:bg-yellow-800 text-gray-600 dark:text-gray-300'
          : notice.level as any === 3
          ? 'bg-orange-100 dark:bg-orange-800 text-gray-600 dark:text-gray-300'
          : notice.level as any === 4
          ? 'bg-red-100 dark:bg-red-800 text-gray-600 dark:text-gray-300'
          : 'bg-red-600 text-white'}`}>
        {notice.local ? <><EyeOutlineIcon class="inline-block mr-1 align-middle" /> Only you can see this.</> : <>
          {/* TODO: name the user, group, or advisory */}
          {notice.relevantUserId ? <span class={chipClasses} onClick={() => invoke("show_user_details", {user: notice.relevantUserId})}>
            <AccountIcon class="w-4 h-4" />User</span> : null}
          {notice.relevantGroupId ? <a class={chipClasses} href={`https://vrchat.com/home/group/${notice.relevantGroupId}`} target="_blank" rel="noopener noreferrer">
            <HexagonMultipleIcon class="w-4 h-4" />Group</a> : null}
          {/* {notice.relevantAdvisoryId ? <span class={chipClasses} onClick={() => invoke("show_advisories_window", {advisory: notice.relevantAdvisoryId})}>
            <FlagIcon class="w-4 h-4" />Advisory</span> : null} */}
        </>}
        <span class="flex-1" />
        {notice.createdAt && <span class="italic text-xs">{new Date(notice.createdAt).toLocaleString()}</span>}
      </div>
    </div>
  );
}