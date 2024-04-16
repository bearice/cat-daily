from zipfile import ZipFile
import json
import sys

path = sys.argv[1]
if not path:
    print(f"usage: {sys.argv[0]} <path to zip file>")
    exit(1)

zf = ZipFile(path)
data = json.loads(zf.open('data/tweets.js').read()[25:])


def is_cat_tweet(tweet):
    tags = [hashtag['text']
            for hashtag in tweet.get('entities').get('hashtags')]
    return "每日一猫" in tags and 'media' in tweet.get('entities', {})


class CatMedia:
    def __init__(self, media):
        self.media_type = media.get('type')
        self.width = media.get('sizes', {}).get('large', {}).get('w')
        self.height = media.get('sizes', {}).get('large', {}).get('h')
        self.image_url = media.get('media_url_https')
        video_info = media.get('video_info', {})
        self.video_url = next(
            (variant.get('url') for variant in sorted(
                video_info.get('variants', []),
                key=lambda x: int(x.get('bitrate', '0')),
                reverse=True)),
            None
        ) if media.get('type') == "video" else None


class CatTweet:
    def __init__(self, tweet):
        self.id = tweet.get('id')
        self.text = tweet.get('full_text')
        self.created_at = tweet.get('created_at')
        extended_entities = tweet.get('extended_entities', {})
        self.media = [
            CatMedia(media).__dict__ for media in extended_entities.get('media', [])
        ] if 'media' in extended_entities else []


cats = filter(is_cat_tweet, map(lambda x: x['tweet'], data))

ret = []
for c in cats:
    ret.append(CatTweet(c).__dict__)

ret.sort(key=lambda x: x['id'], reverse=True)
print(len(ret))
with open("cats.json", "w") as file:
    json.dump(ret, file, indent=4)
