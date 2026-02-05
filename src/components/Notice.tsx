import { Notice as NoticeType } from '@app/bindings/Notice';

export default function Notice({ notice }: { notice: NoticeType }) {
  return (
    <div
      className={`m-2 border rounded-lg ${
        notice.level as any === 0
          ? 'bg-gray-50 dark:bg-gray-900 border-gray-400 dark:border-gray-600 text-gray-800 dark:text-gray-200'
          : notice.level as any === 1
          ? 'bg-yellow-50 dark:bg-yellow-900 border-yellow-400 dark:border-yellow-600 text-gray-800 dark:text-gray-200'
          : notice.level as any === 2
          ? 'bg-orange-50 dark:bg-orange-900 border-orange-400 dark:border-orange-600 text-gray-800 dark:text-gray-200'
          : notice.level as any === 3
          ? 'bg-red-50 dark:bg-red-900 border-red-400 dark:border-red-600 text-gray-800 dark:text-gray-200'
          : notice.level as any === 4
          ? 'bg-purple-50 dark:bg-purple-900 border-purple-400 dark:border-purple-600 text-gray-800 dark:text-gray-200'
          : 'bg-red-500 text-white'
      }`}
    >
      <div class="px-4 py-2">
        {notice.title && <h2 class="text-lg font-semibold mb-2">{notice.title}</h2>}
        <p class="mb-2">{notice.message}</p>
      </div>
      <div class={`rounded-b-lg p-4 text-sm ${notice.level as any === 0
          ? 'bg-gray-100 dark:bg-gray-800 text-gray-600 dark:text-gray-300'
          : notice.level as any === 1
          ? 'bg-yellow-100 dark:bg-yellow-800 text-gray-600 dark:text-gray-300'
          : notice.level as any === 2
          ? 'bg-orange-100 dark:bg-orange-800 text-gray-600 dark:text-gray-300'
          : notice.level as any === 3
          ? 'bg-red-100 dark:bg-red-800 text-gray-600 dark:text-gray-300'
          : notice.level as any === 4
          ? 'bg-purple-100 dark:bg-purple-800 text-gray-600 dark:text-gray-300'
          : 'bg-red-600 text-white'}`}>
        {/* TODO: chips here for relevant user, group, or advisory */}
        {notice.local ? 'Only you can see this.' : 'Visible to everyone.'}
        {/* TODO: show the time the notice was published */}
      </div>
    </div>
  );
}