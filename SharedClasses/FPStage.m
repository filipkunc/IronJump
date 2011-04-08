//
//  FPStage.m
//  IronJump
//
//  Created by Filip Kunc on 8/14/10.
//  For license see LICENSE.TXT
//

#import "FPStage.h"
#import "FPGame.h"

const int MaxStageLevelCount = 11;

FPTexture *menuBackground;
FPTexture *levelBlack;
FPTexture *levelBlue;
FPTexture *levelShine;
FPTexture *arrowButton;
FPTexture *playButton;

CGRect playRect;

int currentLevelCustom;
int currentLevelOriginal;
int maxAvailableLevel;
int currentStage; 

NSMutableArray *stages;

@implementation FPStage

@synthesize isCustom, stageNumber;

+ (void)initStages
{
	menuBackground = [[FPTexture alloc] initWithFile:@"menu.png" convertToAlpha:NO];
	levelBlack = [[FPTexture alloc] initWithFile:@"Level_black.png" convertToAlpha:NO];
	levelBlue = [[FPTexture alloc] initWithFile:@"Level_blue.png" convertToAlpha:NO];
	levelShine = [[FPTexture alloc] initWithFile:@"Level_shine.png" convertToAlpha:NO];
    arrowButton = [[FPTexture alloc] initWithFile:@"arrow.png" convertToAlpha:NO];
	playButton = [[FPTexture alloc] initWithFile:@"play.png" convertToAlpha:NO];
	
	playRect = CGRectMake(80.0f, 325.0f, 160.0f, 57.0f);
	
	stages = [[NSMutableArray alloc] init];
	currentStage = 0;
	currentLevelCustom = [[NSUserDefaults standardUserDefaults] integerForKey:@"currentLevelCustom"];
	currentLevelOriginal = [[NSUserDefaults standardUserDefaults] integerForKey:@"currentLevel"];
	maxAvailableLevel = [[NSUserDefaults standardUserDefaults] integerForKey:@"maxAvailableLevel"];
}

+ (void)saveDefaults
{
	BOOL customStage = [FPStage currentStage].isCustom;
	
	[[NSUserDefaults standardUserDefaults] setInteger:currentLevelCustom forKey:@"currentLevelCustom"];
	[[NSUserDefaults standardUserDefaults] setInteger:currentLevelOriginal forKey:@"currentLevel"];
	[[NSUserDefaults standardUserDefaults] setBool:customStage forKey:@"customStage"];
	[[NSUserDefaults standardUserDefaults] setInteger:maxAvailableLevel forKey:@"maxAvailableLevel"];
}

+ (void)decideCurrentStage
{
	BOOL customStage = [[NSUserDefaults standardUserDefaults] boolForKey:@"customStage"];
	
	int maxCustomStage = 0;
	for (FPStage *stage in stages)
	{
		if (stage.isCustom)
			maxCustomStage++;
		else 
			break;
	}
	
	if (customStage)
	{
		currentStage = currentLevelCustom / MaxStageLevelCount;
		if (currentStage > maxCustomStage)
		{
			currentStage = 0;
			currentLevelCustom = 0;
		}
	}
	else
	{
		currentStage = maxCustomStage + currentLevelOriginal / MaxStageLevelCount;
	}
}

+ (NSUInteger)stageCount
{
	return [stages count];
}

+ (FPStage *)currentStage
{
	return [FPStage stageAtIndex:currentStage];
}

+ (void)addStagesFromLevels:(NSArray *)levelNames isCustom:(BOOL)custom
{
	int stageNumber = 0;
	for (int i = 0; i < levelNames.count; i += MaxStageLevelCount)
	{
		FPStage *stage = [[FPStage alloc] initWithStage:stageNumber isCustom:custom];
		int min = MIN(i + MaxStageLevelCount, levelNames.count);
		for (int j = i; j < min; j++)
		{
			[stage addLevel:(NSString *)[levelNames objectAtIndex:j]];
		}
		[stages addObject:stage];
		[stage release];
		stageNumber++;
	}
}

+ (FPStage *)stageAtIndex:(NSUInteger)index
{
	return (FPStage *)[stages objectAtIndex:index];
}

+ (FPMenuButton)touchEndedAtPoint:(CGPoint)point
{
	CGPoint location;
	location.x = point.y - 80.0f;
	location.y = 320.0f - point.x + 80.0f;
		
	if ([[FPStage currentStage] touchedOnLevelAtLocation:location])
		return FPMenuButtonNone;
	
	if (CGRectContainsPoint(CGRectMake(-72.0f, 192.0f, 64.0f, 128.0f), location))
	{
		currentStage--;
		if (currentStage < 0)
			currentStage = 0;
	}
	else if (CGRectContainsPoint(CGRectMake(393.0f - 64.0f, 192.0f, 64.0f, 128.0f), location))
	{
		currentStage++;
		if (currentStage >= [stages count])
			currentStage = [stages count] - 1;
	}
	else if (CGRectContainsPoint(playRect, location))
	{
		int index = [FPStage currentStage].currentLevelIndex;
		if (index >= 0 && index < [FPStage currentStage].count)
			return FPMenuButtonPlay;
	}
	
	return FPMenuButtonNone;
}

+ (void)drawCurrentStage
{
	[[FPStage currentStage] draw];
}

+ (BOOL)nextLevel
{
	FPStage *stage = [FPStage currentStage];
	if (stage.isCustom)
	{
		currentLevelCustom++;
		if (currentLevelCustom >= stage.stageNumber * MaxStageLevelCount + stage.count)
		{
			currentStage++;
			if (currentStage >= stages.count || ![[FPStage stageAtIndex:currentStage] isCustom])
			{
				currentStage--;
				currentLevelCustom--;
				return NO; // victory
			}
		}
	}
	else
	{
		currentLevelOriginal++;
		if (currentLevelOriginal >= stage.stageNumber * MaxStageLevelCount + stage.count)
		{
			currentStage++;
			if (currentStage >= stages.count)
			{
				currentStage--;
				currentLevelOriginal--;
				return NO; // victory
			}
		}
		if (currentLevelOriginal > maxAvailableLevel)
		{
			maxAvailableLevel = currentLevelOriginal;
			[[NSUserDefaults standardUserDefaults] setInteger:maxAvailableLevel forKey:@"maxAvailableLevel"];
		}
	}
	return YES;
}

- (id)initWithStage:(int)aStage isCustom:(BOOL)custom
{
	self = [super init];
	if (self)
	{
		levels = [[NSMutableArray alloc] init];
		stageNumber = aStage;
		isCustom = custom;
	}
	return self;
}

- (void)dealloc
{
	[levels release];
	[super dealloc];
}

- (NSUInteger)count
{
	return [levels count];
}

- (int)currentLevelIndex
{
	if (isCustom)
		return currentLevelCustom - stageNumber * MaxStageLevelCount;
	return currentLevelOriginal - stageNumber * MaxStageLevelCount;
}

- (void)addLevel:(NSString *)level
{
	[levels addObject:level];
}

- (NSString *)levelAtIndex:(NSUInteger)index
{
	return (NSString *)[levels objectAtIndex:index];
}

- (BOOL)touchedOnLevelAtLocation:(CGPoint)location
{
	for (int i = 0; i < MaxStageLevelCount && i < [levels count]; i++)
	{
		int x = i;
		CGPoint pt;
		if (x < 6)
			pt = CGPointMake(1.0f + x * 55.0f, 210.0f);
		else
			pt = CGPointMake(29.0f + (x - 6) * 55.0f, 265.0f);
		
		CGRect rect = CGRectMake(pt.x, pt.y, 44.0f, 44.0f);
		
		if (CGRectContainsPoint(rect, location))
		{
			int levelNumber = stageNumber * MaxStageLevelCount + i;
			if (isCustom)
			{
				currentLevelCustom = levelNumber;
				return YES;
			}
			else if (levelNumber <= maxAvailableLevel)
			{
				currentLevelOriginal = levelNumber;
				return YES;
			}
			return NO;
		}
	}
	return NO;
}

- (void)draw
{
	FPFont *font = [FPGame font];
	NSString *text;
	
	glDisable(GL_BLEND);
	[menuBackground drawAtPoint:CGPointMake(-80.0f, 80.0f)];
	glEnable(GL_BLEND);
	
	for (int x = 0; x < MaxStageLevelCount && x < [levels count]; x++)
	{
		int levelNumber = stageNumber * MaxStageLevelCount + x;
		CGPoint pt;
		if (x < 6)
			pt = CGPointMake(1.0f + x * 55.0f, 200.0f);
		else
			pt = CGPointMake(29.0f + (x - 6) * 55.0f, 255.0f);
		
		BOOL isCurrentLevel = NO;
		BOOL isAvailable = YES;
		
		if (isCustom)
		{
			if (levelNumber == currentLevelCustom)
				isCurrentLevel = YES;
		}
		else if (levelNumber == currentLevelOriginal)
		{
			isCurrentLevel = YES;
		}
		else if (levelNumber > maxAvailableLevel)
		{
			isAvailable = NO;
		}
		
		if (isCurrentLevel)
			[levelShine drawAtPoint:pt];
		else if (isAvailable)
			[levelBlue drawAtPoint:pt];
		else
			[levelBlack drawAtPoint:pt];
		
		text = [NSString stringWithFormat:@"%i", levelNumber + 1];
		pt.x += 87.0f;
		pt.y -= 75.0f;
		if (levelNumber + 1 < 10)
			pt.x += 7.0f;
		
		if (isCurrentLevel)
			glColor4f(0, 0, 0, 1);
		[font drawText:text atPoint:pt];
		glColor4f(1, 1, 1, 1);
	}
	
	if (self.currentLevelIndex >= 0 && self.currentLevelIndex < self.count)
		[playButton drawAtPoint:playRect.origin];
    
	[arrowButton drawAtPoint:CGPointMake(-72.0f, 182.0f)];
	glPushMatrix();
	glTranslatef(393.0f, 182.0f, 0.0f);
	glScalef(-1.0f, 1.0f, 1.0f);
	[arrowButton draw];
	glPopMatrix();
	
	if (isCustom)
	{
		text = [NSString stringWithFormat:@"Custom Stage %i", stageNumber + 1];
		[font drawText:text atPoint:CGPointMake(150.0f, 75.0f)];
	}
	else
	{
		text = [NSString stringWithFormat:@"Stage %i", stageNumber + 1];
		[font drawText:text atPoint:CGPointMake(195.0f, 75.0f)];
	}
}

@end
