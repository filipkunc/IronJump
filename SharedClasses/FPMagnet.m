//
//  FPMagnet.m
//  IronJumpLevelEditor
//
//  Created by Filip Kunc on 7/4/10.
//  Copyright (c) 2010 Filip Kunc. All rights reserved.
//

#import "FPMagnet.h"
#import "FPPlayer.h"

#if TARGET_OS_MAC
FPTexture *magnetTexture = nil;
#endif

@implementation FPMagnet

@synthesize x, y, widthSegments, isVisible;

#if TARGET_OS_MAC
+ (FPTexture *)loadTextureIfNeeded
{
	if (!magnetTexture)
		magnetTexture = [[FPTexture alloc] initWithFile:@"magnet.png" convertToAlpha:NO];
	return magnetTexture;
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
	return CGRectMake(x, y, 32.0f * widthSegments, 32.0f);
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
	FPPlayer *player =  (FPPlayer *)game.player;
	CGRect playerRect = [player rect];
	CGRect selfRect = [self rect];
	selfRect.origin.y += 32.0f;
	selfRect.size.height += 32.0f * 4;
	selfRect.origin.x += 18.0f;
	selfRect.size.width -= 18.0f * 2.0f;
	if (CGRectIntersectsRect(selfRect, playerRect))
	{
		if (player.moveY < 5.0f)
			player.moveY = flerpf(player.moveY, 5.0f, 0.3f);
	}
}

- (void)draw
{
#if TARGET_OS_IPHONE
	[[FPGameAtlas sharedAtlas] addMagnetAtPoint:CGPointMake(x, y) widthSegments:widthSegments];
#else
	[FPMagnet loadTextureIfNeeded];
	[magnetTexture drawAtPoint:CGPointMake(x, y) widthSegments:widthSegments heightSegments:1];
#endif
}

- (id<FPGameObject>)duplicateWithOffsetX:(float)offsetX offsetY:(float)offsetY
{
	FPMagnet *duplicated = [[[FPMagnet alloc] initWithWidthSegments:widthSegments] autorelease];
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
