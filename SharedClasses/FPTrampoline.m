//
//  FPTrampoline.m
//  IronJump
//
//  Created by Filip Kunc on 6/5/10.
//  Copyright (c) 2010 Filip Kunc. All rights reserved.
//

#import "FPTrampoline.h"
#import "FPTextureArray.h"
#import "FPPlayer.h"

#if TARGET_OS_MAC
FPTextureArray *trampolineTextures = nil;
#endif

@implementation FPTrampoline

@synthesize x, y, widthSegments, isVisible, textureIndex;

#if TARGET_OS_MAC
+ (FPTexture *)loadTextureIfNeeded
{
	if (!trampolineTextures)
	{
		trampolineTextures = [[FPTextureArray alloc] init];
		[trampolineTextures addTexture:@"trampoline01.png"];
		[trampolineTextures addTexture:@"trampoline02.png"];
		[trampolineTextures addTexture:@"trampoline03.png"];
	}
	return [trampolineTextures textureAtIndex:0];
}
#endif

- (id)init
{
	return [self initWithWidthSegments:1];
}

- (id)initWithWidthSegments:(int)aWidthSegments
{
	self = [super init];
	if (self)
	{
		animationCounter = 0;
		textureIndex = 0;
		textureDirection = 1;
		x = 0;
		y = 0;
		widthSegments = aWidthSegments;
        isVisible = YES;
	}
	return self;
}

- (void)dealloc
{
	[super dealloc];
}

- (void)encodeWithCoder:(NSCoder *)aCoder
{
	[aCoder encodeFloat:x forKey:@"x"];
	[aCoder encodeFloat:y forKey:@"y"];
	[aCoder encodeInt:widthSegments forKey:@"widthSegments"];
}

- (id)initWithCoder:(NSCoder *)aDecoder
{
	self = [self init];
	if (self)
	{
		x = [aDecoder decodeFloatForKey:@"x"];
		y = [aDecoder decodeFloatForKey:@"y"];
		if ([aDecoder containsValueForKey:@"widthSegments"])
			widthSegments = [aDecoder decodeIntForKey:@"widthSegments"];
		else
			widthSegments = 1;
	}
	return self;
}

- (CGRect)rect
{
	return CGRectMake(x, y, 64.0f * widthSegments, 32.0f);
}

- (BOOL)isPlatform
{
	return YES;
}

- (BOOL)isMovable
{
	return NO;
}

- (BOOL)isTransparent
{
	return YES;
}

- (void)moveWithX:(float)offsetX y:(float)offsetY
{
	x += offsetX;
	y += offsetY;
}

- (void)updateWithGame:(id<FPGameProtocol>)game
{
	CGRect playerRect = game.player.rect;
	
	if (!CGRectIntersectsRect(playerRect, self.rect))
	{
		playerRect.size.height += tolerance;
		if (CGRectIntersectsRect(playerRect, self.rect))
		{
			FPPlayer *player = (FPPlayer *)[game player];
			player.moveY = 9.5f;
		}
	}
	
	for (id<FPGameObject> gameObject in game.gameObjects)
	{
		if (gameObject.isMovable)
		{
			CGRect gameObjectRect = gameObject.rect;
			gameObjectRect.size.height += tolerance;
			CGRect intersection = CGRectIntersection(gameObjectRect, self.rect);
			if (!CGRectIsEmpty(intersection) && intersection.size.width > 30.0f)
			{
				gameObject.moveY = 8.0f;
			}
		}
	}
	
	if (++animationCounter > 5)
	{
		textureIndex += textureDirection;
		if (textureIndex < 0 || textureIndex >= 2)
		{
			textureIndex -= textureDirection;
			textureDirection = -textureDirection;
		}
		animationCounter = 0;
	}
}

- (void)draw
{
#if TARGET_OS_IPHONE
	[[FPGameAtlas sharedAtlas] addTrampoline:textureIndex atPoint:CGPointMake(x, y) widthSegments:widthSegments heightSegments:1];
#else
	[FPTrampoline loadTextureIfNeeded];
	FPTexture *texture = [trampolineTextures textureAtIndex:textureIndex];
	[texture drawAtPoint:CGPointMake(x, y) widthSegments:widthSegments heightSegments:1];
#endif
}

- (id<FPGameObject>)duplicateWithOffsetX:(float)offsetX offsetY:(float)offsetY
{
	FPTrampoline *duplicated = [[[FPTrampoline alloc] initWithWidthSegments:widthSegments] autorelease];
	[duplicated moveWithX:x + offsetX y:y + offsetY];
	return duplicated;
}

- (void)parseXMLElement:(NSString *)elementName value:(NSString *)value
{
    if ([elementName isEqualToString:@"x"])
        x = [value floatValue];
    else if ([elementName isEqualToString:@"y"])
        y = [value floatValue];
    else if ([elementName isEqualToString:@"widthSegments"])
        widthSegments = [value floatValue];
}

- (void)writeToXML:(FPXMLWriter *)writer
{
    [writer writeElementWithName:@"x" floatValue:x];
    [writer writeElementWithName:@"y" floatValue:y];
    [writer writeElementWithName:@"widthSegments" intValue:widthSegments];
}

@end
